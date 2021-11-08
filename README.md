# xylem
[![GitHub actions](https://github.com/SOF3/xylem/workflows/CI/badge.svg)](https://github.com/SOF3/xylem/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/xylem.svg)](https://crates.io/crates/xylem)
[![crates.io](https://img.shields.io/crates/d/xylem.svg)](https://crates.io/crates/xylem)
[![docs.rs](https://docs.rs/xylem/badge.svg)](https://docs.rs/xylem)
[![GitHub](https://img.shields.io/github/last-commit/SOF3/xylem)](https://github.com/SOF3/xylem)
[![GitHub](https://img.shields.io/github/stars/SOF3/xylem?style=social)](https://github.com/SOF3/xylem)

Xylem is a stateful type conversion framework for Rust.

## Concepts
Xylem provides the [`Xylem`] trait,
which is similar to the [`std::convert::TryFrom`] trait,
but with the following differences:

### Stateful context
[`Xylem::convert`] passes a mutable [`Context`],
enabling stateful operations throughout the conversion process.
See [`Context`] documentation for details.

### Fixed concersion source
Unlike [`std::convert::TryFrom`],
[`Xylem`] takes the conversion source type
as an associated type [`Xylem::From`]
instead of a generic parameter.
This means each type can only be converted
from exactly one other specific type under a given [`Schema`].

### Schemas
[`Xylem`] accepts a type parameter `S` ("schema"),
which acts as a namespace defining the set of conversion rules.
This allows different downstream crates
to define their own conversion rules
without conflicting with each other.
For example, if crate `foo` wants to convert `bool` from `String`
and crate `bar` wants to convert `bool` from `i32`,
they can separately define schema types `foo::Xylem` and `bar::Xylem`,
then separately implement

```rust
impl Xylem<foo::Schema> for bool {
    type From = String;
    // fn convert() omitted
}
impl Xylem<bar::Schema> for bool {
    type From = i32;
    // fn convert() omitted
}
```

Furthermore, since `foo::Schema` and `bar::Schema`
are declared in their own crates,
`Xylem<S>` is not considered a foreign trait,
so implementing custom conversion rules for [`std`] types
will not result in
[error E0220](https://doc.rust-lang.org/error-index.html#E0220) or
[error E0119](https://doc.rust-lang.org/error-index.html#E0119).

To use the default conversion rules defined by xylem,
make the schema implement the [`SchemaExt`] trait.
There is a convenience macro [`declare_schema`] for this:

```rust
xylem::declare_schema!(Schema: xylem::SchemaExt);

// we defined a schema type called `Schema`.
```

It is recommended to use `Schema` as the schema name
and declare it at the crate level,
because the [`Xylem`][xylem_codegen::Xylem] macro
uses `crate::Schema` as the default schema type.

## The `Xylem` macro
Xylem provides a [`Xylem`][xylem_codegen::Xylem] macro,
which derives the corresponding [`Xylem::From`] type
from a struct or enum
by replacing each type with the corresponding [`Xylem::From`] type,
as well as a [`Xylem`] implementation.
See the [`Xylem`][xylem_codegen::Xylem] documentation for details.

Note that the order of fields matters
because xylem type conversion is stateful,
i.e. previous conversions may affect subsequent ones.


## The `id` feature
With the `id` feature enabled,
xylem provides the [`Id`] type,
which is the motivational use case for xylem:
Deserialize a config file that references other fields by string ID,
replace each declaring ID with an integer storing its occurrence order,
and replace each referencing ID with the occurrence order of the declaring ID.

The [`Id`] type takes two generic parameters, `S` and `X`.
The type `S` is just the schema type,
while the type `X` is the subject of identification.
`X` must also implement the [`Identifiable`] trait,
which has an associated type [`Identifiable::Scope`]
used to provide a namespace for the ID.
The declaring [`Id`] field must be declared under `X`,
and `X` must occur as a (transitive) child of the scope.
Further references to the ID of `X`
must occur also as transitive children of the scope,
because the scope is dropped when it completes parsing.

Declaring IDs are marked with the argument `new = true`.
If the ID is to be cross-referenced after the scope drops,
also mark `track = true`.
Referencing IDs do not need to be marked,
but if they serve to import a scope,
they should be marked as `import = true`.

See [tests/id.rs](https://docs.rs/crate/xylem/*/source/tests/id.rs) and
[tests/cross\_id.rs](https://docs.rs/crate/xylem/*/source/tests/cross_id.rs) for example usage.

Note that it is not a design goal for xylem to support lookahead IDs.
Due to the stateful nature of xylem,
IDs are only indexed when the declaration has been scanned.
There is currently no plan to implement multiple passes
to pre-index IDs.
