//! Generates dynamically resolved identifiers.

use core::fmt;
use std::any::{type_name, TypeId};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{AbstractError, Context, Schema, Xylem};

/// An identifier for type `X`.
///
/// The `Id` type works by ensuring
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Id<S: Schema, X: Identifiable<S>> {
    index: u32, // `usize` is avoided due to unclear serialization format.
    _ph:   PhantomData<&'static (S, X)>,
}

impl<S: Schema, X: Identifiable<S>> Id<S, X> {
    /// Creates a new identifier.
    pub fn new(index: usize) -> Self {
        Self { index: index.try_into().expect("Too many identifiers"), _ph: PhantomData }
    }

    /// Returns the index of the identifier.
    pub fn index(&self) -> usize { self.index.try_into().expect("Too many identifiers") }
}

impl<S: Schema, X: Identifiable<S>> fmt::Debug for Id<S, X> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Id({})", self.index) }
}

impl<S: Schema, X: Identifiable<S>> Clone for Id<S, X> {
    fn clone(&self) -> Self { Self { index: self.index, _ph: PhantomData } }
}

impl<S: Schema, X: Identifiable<S>> Copy for Id<S, X> {}

impl<S: Schema, X: Identifiable<S>> Default for Id<S, X> {
    fn default() -> Self { Self { index: 0, _ph: PhantomData } }
}

impl<S: Schema, X: Identifiable<S>> PartialEq for Id<S, X> {
    fn eq(&self, other: &Self) -> bool { self.index == other.index }
}

impl<S: Schema, X: Identifiable<S>> Eq for Id<S, X> {}

impl<S: Schema, X: Identifiable<S>> PartialOrd for Id<S, X> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

impl<S: Schema, X: Identifiable<S>> Ord for Id<S, X> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.index.cmp(&other.index) }
}

impl<S: Schema, X: Identifiable<S>> Hash for Id<S, X> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.index.hash(state); }
}

impl<S: Schema, X: Identifiable<S>> Xylem<S> for Id<S, X> {
    type From = String;
    type Args = IdArgs;

    #[inline]
    fn convert_impl(
        from: Self::From,
        context: &mut <S as Schema>::Context,
        args: Self::Args,
    ) -> Result<Self, <S as Schema>::Error> {
        let index = {
            let counter =
                context.get_mut::<IdCounter<X>, _>(TypeId::of::<X::Scope>(), Default::default);

            if args.new {
                if counter.names.iter().any(|other| other == &from) {
                    return Err(S::Error::new(format_args!("Duplicate ID {}", &from)));
                }
                let index = counter
                    .names
                    .len()
                    .try_into()
                    .expect("More than u32::MAX_VALUE IDs registered");
                counter.names.push(from.clone());
                index
            } else {
                match counter.names.iter().position(|other| other == &from) {
                    Some(index) => {
                        index.try_into().expect("More than u32::MAX_VALUE IDs registered")
                    }
                    None => return Err(S::Error::new(format_args!("Unknown ID {}", &from))),
                }
            }
        };

        let id = Id { index, _ph: PhantomData };

        if args.new {
            let mut new = false;
            let current_id = context.get_mut::<CurrentId, _>(TypeId::of::<X>(), || {
                new = true;
                CurrentId {
                    id:     id.index(),
                    parent: TypeId::of::<X::Scope>(),
                    string: from.clone(),
                }
            });
            if !new {
                return Err(S::Error::new(format_args!(
                    "Multiple new IDs defined for {} ({}, {})",
                    type_name::<X>(),
                    id.index(),
                    current_id.id,
                )));
            }

            if args.track {
                let mut parent_ids = Vec::new();

                let mut next_parent = TypeId::of::<X::Scope>();
                while let Some(parent_id) = context.get::<CurrentId>(next_parent) {
                    parent_ids.push(parent_id.id);
                    next_parent = parent_id.parent;
                }

                parent_ids.reverse();

                let store =
                    context.get_mut::<GlobalIdStore<S, X>, _>(TypeId::of::<()>(), Default::default);
                store.ids.entry(parent_ids).or_default().push(from);
            }
        }

        Ok(id)
    }
}

/// Arguments for [`Id`].
#[derive(Default)]
pub struct IdArgs {
    /// Whether to generate a new identifier.
    ///
    /// If set to `true`, expects the value to be a new identifier in the namespace.
    /// If set to `false`, expects the value to be an existing identifier in the namespace.
    pub new: bool,

    /// Whether to track the identifier in the root scope.
    ///
    /// The identifier will persist with respect to the unique identifier of the parent.
    /// containing the identifiers of all ancestors (i.e. `X::Scope`, `X::Scope::Scope`, etc.).
    ///
    /// This option is only valid when `new` is `true`,
    /// and cannot be used if the type recurses.
    pub track: bool,
}

/// Retrieves the original string ID for an identifiable object.
pub struct IdString<S: Schema, X: Identifiable<S>> {
    value: String,
    _ph:   PhantomData<&'static (S, X)>,
}

impl<S: Schema, X: Identifiable<S>> IdString<S, X> {
    pub fn value(&self) -> &str { &self.value }
}

impl<S: Schema, X: Identifiable<S>> Xylem<S> for IdString<S, X> {
    type From = ();
    type Args = ();

    #[inline]
    fn convert_impl(
        (): Self::From,
        context: &mut <S as Schema>::Context,
        _args: Self::Args,
    ) -> Result<Self, <S as Schema>::Error> {
        let id = match context.get::<CurrentId>(TypeId::of::<X>()) {
            Some(id) => id,
            None => {
                return Err(S::Error::new(format_args!("No current ID for {}", type_name::<X>())))
            }
        };

        Ok(Self { value: id.string.clone(), _ph: PhantomData })
    }
}

/// Tracks the list of IDs in a scope.
struct IdCounter<X: 'static> {
    names: Vec<String>,
    _ph:   PhantomData<&'static X>,
}

impl<X: 'static> IdCounter<X> {}

impl<X: 'static> Default for IdCounter<X> {
    fn default() -> Self { Self { names: Vec::new(), _ph: PhantomData } }
}

/// Tracks the current ID.
struct CurrentId {
    /// The index of the current identifier.
    ///
    /// This does not use the `Id` type to avoid type parameters.
    id:     usize,
    /// The type ID of the parent.
    parent: TypeId,
    /// The original string ID.
    string: String,
}

struct GlobalIdStore<S: Schema, X: Identifiable<S>> {
    ids: BTreeMap<Vec<usize>, Vec<String>>,
    _ph: PhantomData<&'static (S, X)>,
}

impl<S: Schema, X: Identifiable<S>> Default for GlobalIdStore<S, X> {
    fn default() -> Self { Self { ids: BTreeMap::new(), _ph: PhantomData } }
}

/// A trait for types that can be identified.
pub trait Identifiable<S: Schema>: Xylem<S> {
    /// The scope of the identifier namespace.
    ///
    /// Use `()` for global identifiers.
    type Scope: Xylem<S>;

    /// Returns the identifier for this instance.
    fn id(&self) -> Id<S, Self>;
}
