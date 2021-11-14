//! # xylem
//! [![GitHub actions](https://github.com/SOF3/xylem/workflows/CI/badge.svg)](https://github.com/SOF3/xylem/actions?query=workflow%3ACI)
//! [![crates.io](https://img.shields.io/crates/v/xylem.svg)](https://crates.io/crates/xylem)
//! [![crates.io](https://img.shields.io/crates/d/xylem.svg)](https://crates.io/crates/xylem)
//! [![docs.rs](https://docs.rs/xylem/badge.svg)](https://docs.rs/xylem)
//! [![GitHub](https://img.shields.io/github/last-commit/SOF3/xylem)](https://github.com/SOF3/xylem)
//! [![GitHub](https://img.shields.io/github/stars/SOF3/xylem?style=social)](https://github.com/SOF3/xylem)
//!
//! Xylem is a stateful type conversion framework for Rust.
//!
//! ## Concepts
//! Xylem provides the [`Xylem`] trait,
//! which is similar to the [`std::convert::TryFrom`] trait,
//! but with the following differences:
//!
//! ### Stateful context
//! [`Xylem::convert`] passes a mutable [`Context`],
//! enabling stateful operations throughout the conversion process.
//! See [`Context`] documentation for details.
//!
//! ### Fixed concersion source
//! Unlike [`std::convert::TryFrom`],
//! [`Xylem`] takes the conversion source type
//! as an associated type [`Xylem::From`]
//! instead of a generic parameter.
//! This means each type can only be converted
//! from exactly one other specific type under a given [`Schema`].
//!
//! ### Schemas
//! [`Xylem`] accepts a type parameter `S` ("schema"),
//! which acts as a namespace defining the set of conversion rules.
//! This allows different downstream crates
//! to define their own conversion rules
//! without conflicting with each other.
//! For example, if crate `foo` wants to convert `bool` from `String`
//! and crate `bar` wants to convert `bool` from `i32`,
//! they can separately define schema types `foo::Xylem` and `bar::Xylem`,
//! then separately implement
//!
//! ```ignore
//! impl Xylem<foo::Schema> for bool {
//!     type From = String;
//!     // fn convert() omitted
//! }
//! impl Xylem<bar::Schema> for bool {
//!     type From = i32;
//!     // fn convert() omitted
//! }
//! ```
//!
//! Furthermore, since `foo::Schema` and `bar::Schema`
//! are declared in their own crates,
//! `Xylem<S>` is not considered a foreign trait,
//! so implementing custom conversion rules for [`std`] types
//! will not result in
//! [error E0220](https://doc.rust-lang.org/error-index.html#E0220) or
//! [error E0119](https://doc.rust-lang.org/error-index.html#E0119).
//!
//! To use the default conversion rules defined by xylem,
//! make the schema implement the [`SchemaExt`] trait.
//! There is a convenience macro [`declare_schema`] for this:
//!
//! ```rust
//! xylem::declare_schema!(Schema: xylem::SchemaExt);
//!
//! // we defined a schema type called `Schema`.
//! ```
//!
//! It is recommended to use `Schema` as the schema name
//! and declare it at the crate level,
//! because the [`Xylem`][xylem_codegen::Xylem] macro
//! uses `crate::Schema` as the default schema type.
//!
//! ## The `Xylem` macro
//! Xylem provides a [`Xylem`][xylem_codegen::Xylem] macro,
//! which derives the corresponding [`Xylem::From`] type
//! from a struct or enum
//! by replacing each type with the corresponding [`Xylem::From`] type,
//! as well as a [`Xylem`] implementation.
//! See the [`Xylem`][xylem_codegen::Xylem] documentation for details.
//!
//! Note that the order of fields matters
//! because xylem type conversion is stateful,
//! i.e. previous conversions may affect subsequent ones.
//!
//!
//! ## The `id` feature
//! With the `id` feature enabled,
//! xylem provides the [`Id`] type,
//! which is the motivational use case for xylem:
//! Deserialize a config file that references other fields by string ID,
//! replace each declaring ID with an integer storing its occurrence order,
//! and replace each referencing ID with the occurrence order of the declaring ID.
//!
//! The [`Id`] type takes two generic parameters, `S` and `X`.
//! The type `S` is just the schema type,
//! while the type `X` is the subject of identification.
//! `X` must also implement the [`Identifiable`] trait,
//! which has an associated type [`Identifiable::Scope`]
//! used to provide a namespace for the ID.
//! The declaring [`Id`] field must be declared under `X`,
//! and `X` must occur as a (transitive) child of the scope.
//! Further references to the ID of `X`
//! must occur also as transitive children of the scope,
//! because the scope is dropped when it completes parsing.
//!
//! Declaring IDs are marked with the argument `new = true`.
//! If the ID is to be cross-referenced after the scope drops,
//! also mark `track = true`.
//! Referencing IDs do not need to be marked,
//! but if they serve to import a scope,
//! they should be marked as `import = true`.
//!
//! See [tests/id.rs](https://docs.rs/crate/xylem/*/source/tests/id.rs) and
//! [tests/cross\_id.rs](https://docs.rs/crate/xylem/*/source/tests/cross_id.rs) for example usage.
//!
//! Note that it is not a design goal for xylem to support lookahead IDs.
//! Due to the stateful nature of xylem,
//! IDs are only indexed when the declaration has been scanned.
//! There is currently no plan to implement multiple passes
//! to pre-index IDs.

use std::any::TypeId;
use std::fmt;

// An internal re-export used for reusing arguments.
#[doc(hidden)]
pub use lazy_static::lazy_static;
/// Derives a [`Xylem`] implementation for a struct or enum
/// and the corresponding [`Xylem::From`] type.
///
/// In this page, "input" refers to the struct/enum written by the user manually,
/// and "derived" refers to the struct/enum generated by the macro
/// to be used as the [`Xylem::From`] type.
///
/// The derived type has the same structure as the input type
/// with each field type `Type` replaced by `<Type as Xylem<S>>::From`,
/// except otherwise specified by the field attributes.
/// Conversion takes place by mapping each field of the derived type
/// to the corresponding field of the input type
/// by calling [`Xylem::convert`].
///
/// # Container Attributes
/// The following attributes can be applied on the input.
///
/// ## `#[xylem(expose = Ident)]`
/// Expose the derived type with the specified name `Ident`.
/// The type has the same visibility as the input type.
///
/// ## `#[xylem(schema = path::to::Schema)]`
/// Specify the schema type to define for as `path::to::Schema`.
///
/// ## `#[xylem(serde(xxx))]`
/// Apply the serde attribute `xxx` to the derived type.
///
/// ## `#[xylem(derive(Foo, Bar))]`
/// Apply the derive macros `Foo`, `Bar` on the derived type.
///
/// ## `#[xylem(process)]`
/// Call [`Processable::preprocess`] before conversion,
/// call [`Processable::postprocess`] after conversion.
///
/// Requires the input type to implement the [`Processable`] trait.
///
/// # Field Attributes
/// The following attributes can be applied on the fields in the input.
/// As above, "input field" refers to the field written by the user manually,
/// and "derived field" refers to the field generated by the macro
/// that occur in the derived type.
/// Furthermore, the following assumes that
/// the attribute is applied on `foo: Bar` unless otherwise specified.
///
/// ## `#[xylem(serde(xxx))]`
/// Apply the serde attribute `xxx` to the derived field.
///
/// ## `#[xylem(preserve)]`:
/// The derived field will use the same type as the input field,
/// i.e. `#[xylem(preserve)] foo: Bar` generats `foo: Bar` in the derived type.
/// The value in the derived field is directly moved to the target field.
///
/// ## `#[xylem(transform = path(Type))]`
/// Generate a field `foo: Type` in the derived type,
/// and call `path(derived.foo)` during conversion.
/// `path` is the path to a function with the signature
/// `fn(Type) -> Result<Bar, S::Error>`.
/// For example, `#[xylem(transform = Ok(Type))]`
/// is equivalent to `#[xylem(preserve)]`.
///
/// ## `#[xylem(transform_with_context = path(Type))]`
/// Similar to `transform`, except `path` accepts an extra context parameter,
/// giving the signature `fn(Type, &mut S::Context) -> Result<Field, S::Error>`.
///
/// ## `#[xylem(default = expr)]`
/// Always uses `expr` (resolved every time the struct
/// or the enum variant is constructed) as the value.
/// Does not generate a field in the `From` type.
/// The expression should have type `Result<Field, S::Error>`,
/// where `Field` is the field type.
///
/// Comparing `default`, `preserve`, `transform` and `transform_with_context`:
/// - If a corresponding field is required in the derived type,
///     - If they have different types,
///         - If context is required, use `transform_with_context`.
///         - Otherwise, use `transform`.
///     - Otherwise, use `preserve`.
/// - Otherwise, use `default`.
///
/// ## `#[xylem(args(key1 = value1, key2 = value2))]`
/// Pass the given arguments in the [`Xylem::convert`] call.
/// Incompatible with `default`, `preserve`, `transform` and `transform_with_context`.
/// `key1` and `key2` are visible named fields in `<Bar as Xylem<S>>::Args`.
/// The values in the key are evaluated lazily and stored as a `static`.
/// The generated code is equivalent to the following:
///
/// ```ignore
/// lazy_static! {
///     static ref ARGS: Args = Args {
///         key1: value1,
///         key2: value2,
///     };
/// }
/// <Bar as Xylem<S>>::convert(derived.foo, context, &*ARGS)
/// ```
pub use xylem_codegen::Xylem;

#[cfg(feature = "id")]
mod id;
#[cfg(feature = "id")]
pub use id::{Id, IdArgs, IdString, Identifiable};
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
        args: &Self::Args,
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
        args: &Self::Args,
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
        _: &Self::Args,
    ) -> Result<Self, <S as Schema>::Error> {
        Ok(())
    }
}

/// Preprocessor and postprocessor extensions for [`Xylem`].
pub trait Processable<S: Schema + ?Sized>: Xylem<S> {
    /// This method is called at the beginning of [`Xylem::convert_impl`] if `#[xylem(process)]` is
    /// provided.
    fn preprocess(
        _from: &mut <Self as Xylem<S>>::From,
        _context: &mut <S as Schema>::Context,
    ) -> Result<(), <S as Schema>::Error> {
        Ok(())
    }

    /// This method is called just before [`Xylem::convert_impl`] returns if `#[xylem(process)]` is
    /// provided.
    fn postprocess(
        &mut self,
        _context: &mut <S as Schema>::Context,
    ) -> Result<(), <S as Schema>::Error> {
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
    /// Identifies a layer of scope.
    type Scope;

    /// Gets the nth topmost scope type ID.
    fn nth_last_scope(&self, n: usize) -> Option<TypeId>;

    /// Gets a shared reference to the storage of type `T`
    /// in the newest layer of the scope.
    fn get<T>(&self, scope: TypeId) -> Option<&T>
    where
        T: 'static;

    /// Gets a shared reference to the storage of type `T`
    /// in each layer, from top to bottom, if exists.
    fn get_each<T>(&self) -> Box<dyn Iterator<Item = &T> + '_>
    where
        T: 'static;

    /// Gets a mutable reference to the storage of type `T`
    /// in the newest layer of the scope.
    fn get_mut<T, F>(&mut self, scope: TypeId, default: F) -> &mut T
    where
        F: FnOnce() -> T,
        T: 'static;

    /// Pushes the type to the scope stack.
    ///
    /// This method is automatically called
    /// from [`Xylem::convert`].
    fn start_scope<T: 'static>(&mut self) -> Self::Scope;

    /// Pops a type from the scope stack.
    ///
    /// This method is automatically called
    /// from [`Xylem::convert`].
    fn end_scope(&mut self, scope: Self::Scope);
}

/// The default empty argument type.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoArgs;

#[cfg(feature = "typemap")]
mod typemap_context;
#[cfg(feature = "typemap")]
pub use typemap_context::DefaultContext;

/// Declare a normal schema type.
///
/// # Example
/// ```
/// xylem::declare_schema!(MySchema);
///
/// #[derive(xylem::Xylem)]
/// #[xylem(schema = MySchema)]
/// struct Foo {}
/// ```
#[macro_export]
macro_rules! declare_schema {
    ($(#[$meta:meta])* $vis:vis $name:ident $(: $($traits:path),+)?) => {
        $(#[$meta])*
        $vis enum $name {}

        impl $crate::Schema for $name {
            type Context = $crate::DefaultContext;
            type Error = anyhow::Error;
        }

        $($(
            impl $traits for $name {}
        )*)?
    }
}
