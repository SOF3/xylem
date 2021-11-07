# xylem

A stateful type conversion framework for Rust.

<details>
<summary>Example use case</summary>

We are loading a huge config file with many references.

```toml
[[building]]
id = "house"
  [[building.inlet]]
  id = "electricity"

[[building]]
id = "power-plant"
  [[building.outlet]]
  id = "power"

[[pipe]]
  [pipe.from]
  building = "power-plant"
  outlet = "power"

  [pipe.to]
  building = "house"
  inlet = "electricity"
```

Let's parse this config with serde:

```rust
#[derive(Deserialize)]
struct Building {
  id: String,
  #[serde(default)] inlet: Vec<Inlet>,
  #[serde(default)] outlet: Vec<Inlet>,
  // other fields...
}
#[derive(Deserialize)]
struct Inlet {
  id: String,
  // other fields...
}
#[derive(Deserialize)]
struct Outlet {
  id: String,
  // other fields...
}

#[derive(Deserialize)]
struct Pipe {
  from: PipeFrom,
  to: PipeTo,
  // other fields...
}
#[derive(Deserialize)]
struct PipeFrom {
  building: String,
  outlet: String,
}
#[derive(Deserialize)]
struct PipeTo {
  building: String,
  inlet: String,
}
```

But we are indexing objects with strings,
which is bad for runtime performance.
Ideally, we can convert our IDs to integers
representing the index of the object in the list.
However, this conversion requires stateful deserialization,
which is not possible with serde.
Writing the conversion code by hand is very boring
because we would need to maintain two copies for each data type
and include a boilerplate for the conversion.

Xylem provides a framework to convert from the human-friendly strings
to the runtime integers by passing a stateful context that tracks the IDs.

</details>

## Concepts

Xylem provides a trait called [`Xylem`](https://docs.rs/xylem/*/xylem/trait.Xylem.html),
which is similar to the `From` trait in the standard library.
But in addition to the source value, it also provides a `Context` and an `Args`,
allowing implementors to pass data during the conversion process.

Unlike the `From` trait,
the `Xylem` trait takes the source type as an associated type instead of a type parameter.
This means each type can only be converted from another specific type.
This allows the `Xylem` derive macro to generate identical structs/enums
that contain the respective source types for each field.
However, the `Xylem` trait has a "schema" parameter,
which expects a type declared in the user crate.
Thus, all conversions are only defined under the schema scope,
allowing multiple users to define their own sets of conversion rules,
even for standard library types.

`Context` provides a stack of scopes,
each of which store a typemap of data,
allowing special `Xylem` implementors to store their own states.
For example, the `id` feature uses the context to track the IDs that have appeared in the conversions.


