use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

use crate::{Schema, Xylem};

/// Implement this trait for a schema type to implement "standard" conversions.
///
/// See the [`SchemaExt#implementors`] for the list of standard conversions included.
pub trait SchemaExt: Schema {}

impl<T: SchemaExt> OptionSchemaExt for T {}

/// Implement this trait for a schema type to support standard box conversion.
///
/// This allows `Box<T>` to be converted from `Box<T::From>`,
/// basically making the `Box` wrapper transparent.
/// The argument type is passed as-is.
pub trait BoxSchemaExt: Schema {}

impl<S: BoxSchemaExt, T: Xylem<S>> Xylem<S> for Box<T> {
    type From = Box<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: &Self::Args,
    ) -> Result<Self, S::Error> {
        Ok(Box::new(T::convert(*from, context, args)?))
    }
}

impl<T: SchemaExt> BoxSchemaExt for T {}

/// Implement this trait for a schema type to support standard [`Rc`] conversion.
///
/// This allows `Rc<T>` to be converted from `Box<T::From>`,
/// basically making the `Rc` wrapper transparent.
/// The argument type is passed as-is.
///
/// # Choice of the `From` associated type
/// The choice of `Box` instead of `Rc` in the `From` type
/// considers the design goal where
/// the `From` type is typically produced from deserialization directly,
/// where the `Rc` wrapper is just a `Box` with some more overhead,
/// where the underlying type cannot be moved out without a panic branch,
/// in which case a `Box` would have just been more reasonable.
///
/// The reason for using `Box<T>` instead of `T`
/// considers the possibility where `T` is a recursive type.
pub trait RcSchemaExt: Schema {}

impl<S: RcSchemaExt, T: Xylem<S>> Xylem<S> for Rc<T> {
    type From = Box<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: &Self::Args,
    ) -> Result<Self, S::Error> {
        Ok(Rc::new(T::convert(*from, context, args)?))
    }
}

impl<T: SchemaExt> RcSchemaExt for T {}

/// Implement this trait for a schema type to support standard [`Arc`] conversion.
///
/// This allows `Arc<T>` to be converted from `Arc<T::From>`,
/// basically making the `Arc` wrapper transparent.
/// The argument type is passed as-is.
///
/// # Choice of the `From` associated type
/// The choice of `Box` instead of `Arc` in the `From` type
/// considers the design goal where
/// the `From` type is typically produced from deserialization directly,
/// where the `Arc` wrapper is just a `Box` with some more overhead,
/// where the underlying type cannot be moved out without a panic branch,
/// in which case a `Box` would have just been more reasonable.
///
/// The reason for using `Box<T>` instead of `T`
/// considers the possibility where `T` is a recursive type.
pub trait ArcSchemaExt: Schema {}

impl<S: ArcSchemaExt, T: Xylem<S>> Xylem<S> for Arc<T> {
    type From = Box<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: &Self::Args,
    ) -> Result<Self, S::Error> {
        Ok(Arc::new(T::convert(*from, context, args)?))
    }
}

impl<T: SchemaExt> ArcSchemaExt for T {}

/// Implement this trait for a schema type to support standard [`Option`] conversion.
///
/// This allows `Option<T>` to be converted from `Option<T::From>`,
/// using the conversion for `T` if it is a `Some`, preserving `None` otherwise.
/// The argument type is passed as-is.
pub trait OptionSchemaExt: Schema {}

impl<S: OptionSchemaExt, T: Xylem<S>> Xylem<S> for Option<T> {
    type From = Option<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: &Self::Args,
    ) -> Result<Self, S::Error> {
        Ok(match from {
            Some(from) => Some(T::convert(from, context, args)?),
            None => None,
        })
    }
}

/// Implement this trait for a schema type to support standard [`Vec`] conversion.
///
/// This allows `Vec<T>` to be converted from `Vec<T::From>`,
/// applying the conversion for `T` elementwise.
/// The argument is passed as-is for each element.
pub trait VecSchemaExt: Schema {}

impl<S: VecSchemaExt, T: Xylem<S>> Xylem<S> for Vec<T> {
    type From = Vec<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: &Self::Args,
    ) -> Result<Self, S::Error> {
        from.into_iter().map(|item| T::convert(item, context, args)).collect()
    }
}

impl<T: SchemaExt> VecSchemaExt for T {}

/// Implement this trait for a schema type to support standard [`HashMap`] conversion.
///
/// This allows `HashMap<K, V>` to be converted from `HashMap<K::From, V::From>`,
/// applying the conversion for `K` for k.
/// The value argument is passed as-is for each value.
/// No conversion arguments can be passed to the key type (the default value is always used).
pub trait HashMapSchemaExt: Schema {}

impl<S: HashMapSchemaExt, K: Xylem<S>, V: Xylem<S>> Xylem<S> for HashMap<K, V>
where
    K: Eq + Hash,
    K::From: Eq + Hash,
    <V as Xylem<S>>::Args: Default,
{
    type From = HashMap<K::From, V::From>;
    type Args = <V as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: &Self::Args,
    ) -> Result<Self, S::Error> {
        from.into_iter()
            .map(|(key, value)| {
                Ok((
                    K::convert(key, context, &Default::default())?,
                    V::convert(value, context, args)?,
                ))
            })
            .collect()
    }
}

impl<T: SchemaExt> HashMapSchemaExt for T {}

/// Implement this trait for a schema type to support standard [`HashMap`] conversion.
///
/// This allows `HashMap<K, V>` to be converted from `HashMap<K::From, V::From>`,
/// applying the conversion for `K` for k.
/// The value argument is passed as-is for each value.
/// No conversion arguments can be passed to the key type (the default value is always used).
pub trait BTreeMapSchemaExt: Schema {}

impl<S: BTreeMapSchemaExt, K: Xylem<S>, V: Xylem<S>> Xylem<S> for BTreeMap<K, V>
where
    K: Eq + Ord,
    K::From: Eq + Ord,
    <V as Xylem<S>>::Args: Default,
{
    type From = BTreeMap<K::From, V::From>;
    type Args = <V as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: &Self::Args,
    ) -> Result<Self, S::Error> {
        from.into_iter()
            .map(|(key, value)| {
                Ok((
                    K::convert(key, context, &Default::default())?,
                    V::convert(value, context, args)?,
                ))
            })
            .collect()
    }
}

impl<T: SchemaExt> BTreeMapSchemaExt for T {}
