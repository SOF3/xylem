use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crate::{Schema, Xylem};

pub trait SchemaExt: Schema {}

impl<T: SchemaExt> OptionSchemaExt for T {}

pub trait BoxSchemaExt: Schema {}

impl<S: BoxSchemaExt, T: Xylem<S>> Xylem<S> for Box<T> {
    type From = Box<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: Self::Args,
    ) -> Result<Self, S::Error> {
        Ok(Box::new(T::convert(*from, context, args)?))
    }
}

impl<T: SchemaExt> BoxSchemaExt for T {}

pub trait OptionSchemaExt: Schema {}

impl<S: OptionSchemaExt, T: Xylem<S>> Xylem<S> for Option<T> {
    type From = Option<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: Self::Args,
    ) -> Result<Self, S::Error> {
        Ok(match from {
            Some(from) => Some(T::convert(from, context, args)?),
            None => None,
        })
    }
}

pub trait VecSchemaExt: Schema {}

impl<S: VecSchemaExt, T: Xylem<S>> Xylem<S> for Vec<T>
where
    <T as Xylem<S>>::Args: Default + Clone,
{
    type From = Vec<T::From>;
    type Args = <T as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: Self::Args,
    ) -> Result<Self, S::Error> {
        from.into_iter().map(|item| T::convert(item, context, args.clone())).collect()
    }
}

impl<T: SchemaExt> VecSchemaExt for T {}

pub trait HashMapSchemaExt: Schema {}

impl<S: HashMapSchemaExt, K: Xylem<S>, V: Xylem<S>> Xylem<S> for HashMap<K, V>
where
    K: Eq + Hash,
    K::From: Eq + Hash,
    <V as Xylem<S>>::Args: Default + Clone,
{
    type From = HashMap<K::From, V::From>;
    type Args = <V as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: Self::Args,
    ) -> Result<Self, S::Error> {
        from.into_iter()
            .map(|(key, value)| {
                Ok((
                    K::convert(key, context, Default::default())?,
                    V::convert(value, context, args.clone())?,
                ))
            })
            .collect()
    }
}

impl<T: SchemaExt> HashMapSchemaExt for T {}

pub trait BTreeMapSchemaExt: Schema {}

impl<S: BTreeMapSchemaExt, K: Xylem<S>, V: Xylem<S>> Xylem<S> for BTreeMap<K, V>
where
    K: Eq + Ord,
    K::From: Eq + Ord,
    <V as Xylem<S>>::Args: Default + Clone,
{
    type From = BTreeMap<K::From, V::From>;
    type Args = <V as Xylem<S>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut S::Context,
        args: Self::Args,
    ) -> Result<Self, S::Error> {
        from.into_iter()
            .map(|(key, value)| {
                Ok((
                    K::convert(key, context, Default::default())?,
                    V::convert(value, context, args.clone())?,
                ))
            })
            .collect()
    }
}

impl<T: SchemaExt> BTreeMapSchemaExt for T {}
