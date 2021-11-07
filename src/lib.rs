//! Xylem is a framework for context-sensitive type conversion.
//!
//! To be specific, xylem is used in the case where
//! some data are more efficiently represented during runtime as the type `T`,
//! but they are loaded from configuration files as the type `U`.
//! `S` is deserialized with serde directly,
//! but the conversion from `U` to `T` is stateful in the nested types.
//! In this case, we implement `T: xylem::Xylem<From = U>`.
//!
//! The most general example is to
//! cross-reference another item by a string ID in the configuration file
//! so that readers and editors can interpret it more easily,
//! but identifying by an integer ID is more efficient at runtime.
//!
//! ## Compatibility
//! To allow compatibility of xylem usages in different cases,
//! the `Xylem` trait accepts a type parameter `S`,
//! which is a type defined in your own crate.
//! It represents the schema of this conversion,
//! so `S -> T` rules are all namespaced under the type `T`.
//! This avoids [error E0119](https://doc.rust-lang.org/error-index.html#E0119)
//! due to different conversion rules in different schemas.

use std::any::TypeId;
use std::fmt;

pub use xylem_codegen::Xylem;

#[cfg(feature = "id")]
pub mod id;
#[cfg(feature = "id")]
pub use id::Id;
#[cfg(feature = "ext")]
mod ext;
#[cfg(feature = "ext")]
pub use ext::*;

/// Implementors of this trait have a special conversion rule under the schema `Schema`.
pub trait Xylem<S: Schema + ?Sized>: Sized + 'static {
    /// The type to convert from.
    type From: Sized;

    /// The args provided in the field.
    ///
    /// The type must be a struct that implements [`Default`],
    /// allowing the macro to instantiate it in the following format:
    ///
    /// ```ignore
    /// Args {
    ///    key1: value1,
    ///    key2: value2,
    ///    ..Default::default()
    /// }
    /// ```
    ///
    /// where the macro has the field attribute `#[xylem(key1 = value1, key2 = value2)]`.
    type Args: Default;

    /// Converts the `From` type to the `Self` type,
    /// registering the scope with the context.
    /// Do not override this method.
    #[inline]
    fn convert(
        from: Self::From,
        context: &mut <S as Schema>::Context,
        args: Self::Args,
    ) -> Result<Self, <S as Schema>::Error> {
        let scope = context.start_scope::<Self>();
        let ret = Self::convert_impl(from, context, args)?;
        context.end_scope(scope);
        Ok(ret)
    }

    /// The implementation of the conversion.
    fn convert_impl(
        from: Self::From,
        context: &mut <S as Schema>::Context,
        args: Self::Args,
    ) -> Result<Self, <S as Schema>::Error>;
}

/// The unit type is used as the dummy type for the root scope.
impl<S> Xylem<S> for ()
where
    S: Schema,
{
    type From = ();
    type Args = NoArgs;

    fn convert_impl(
        _: Self::From,
        _: &mut <S as Schema>::Context,
        _: Self::Args,
    ) -> Result<Self, <S as Schema>::Error> {
        Ok(())
    }
}

/// The schema type for a specific set of conversion rules.
///
/// Implementors should be declared in the same crate as the type they convert
/// to avoid [error E0210](https://doc.rust-lang.org/error-index.html#E0210).
pub trait Schema: 'static {
    /// The context type for this schema.
    type Context: Context;

    /// The error type for conversions in this schema.
    type Error: AbstractError;
}

/// The error type for a schema.
pub trait AbstractError: Sized {
    /// Creates a new error type.
    fn new<T: fmt::Display>(msg: T) -> Self;
}

#[cfg(feature = "anyhow")]
impl AbstractError for anyhow::Error {
    fn new<T: fmt::Display>(msg: T) -> Self { anyhow::anyhow!("{}", msg) }
}

/// The context of a conversion.
///
/// The context provides a stack of scopes.
/// A scope is typically the duration of the conversion of a value,
/// and the stack grows when types are converted recursively.
/// The top of the stack is the current scope.
///
/// Each layer of the stack has its own typemap,
/// which provides access to an arbitrary object bound accessible during the scope.
///
/// It is strongly discouraged to have recursive types
/// resulting in multiple layers of the scope to have the same type ID,
/// which may need to strange behavior when accessing the typemap,
/// becaues only the newest layer of the type is accessible.
pub trait Context: Default {
    type Scope;

    /// Gets a shared reference to the storage of type `T`
    /// in the newest layer of `S`.
    fn get<T>(&self, scope: TypeId) -> Option<&T>
    where
        T: 'static;

    /// Gets a mutable reference to the storage of type `T`
    /// in the newest layer of `S`.
    fn get_mut<T, F>(&mut self, scope: TypeId, default: F) -> &mut T
    where
        F: FnOnce() -> T,
        T: 'static;

    /// Pushes the type to the scope stack.
    /// Dropping the return value ends the scope.
    fn start_scope<T: 'static>(&mut self) -> Self::Scope;

    /// Pops a type from the scope stack.
    /// Dropping the return value ends the scope.
    fn end_scope(&mut self, scope: Self::Scope);
}

/// The default empty argument type.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoArgs;

#[cfg(feature = "typemap")]
pub mod typemap_context;
#[cfg(feature = "typemap")]
pub use typemap_context::DefaultContext;
