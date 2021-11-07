use xylem::id::Identifiable;
use xylem::{DefaultContext, Id, NoArgs, SchemaExt, Xylem};

enum Schema {}

impl xylem::Schema for Schema {
    type Context = DefaultContext;

    type Error = anyhow::Error;
}

impl SchemaExt for Schema {}

// Test for cross-referencing IDs

#[derive(Debug, xylem::Xylem)]
#[xylem(expose)]
struct Foo {
    #[xylem(args(new = true))]
    id:    Id<Schema, Foo>,
    other: Option<Id<Schema, Foo>>,
    bar:   Vec<Bar>,
}

impl Identifiable<Schema> for Foo {
    type Scope = ();

    fn id(&self) -> Id<Schema, Self> { self.id }
}

#[test]
fn test_cross_ref() {
    let mut context = DefaultContext::default();

    let first = Foo::convert(
        FooXylem { id: String::from("first"), other: None, bar: Vec::new() },
        &mut context,
        NoArgs,
    )
    .unwrap();

    assert_eq!(first.id.index(), 0);
    assert!(first.other.is_none());

    let second = Foo::convert(
        FooXylem { id: String::from("second"), other: None, bar: Vec::new() },
        &mut context,
        NoArgs,
    )
    .unwrap();

    assert_eq!(second.id.index(), 1);
    assert!(second.other.is_none());

    let third = Foo::convert(
        FooXylem {
            id:    String::from("third"),
            other: Some(String::from("first")),
            bar:   Vec::new(),
        },
        &mut context,
        NoArgs,
    )
    .unwrap();

    assert_eq!(third.id.index(), 2);
    assert!(match third.other {
        Some(id) => id.index() == 0,
        None => false,
    });
}

// Test for scoped IDs

#[derive(Debug, xylem::Xylem)]
#[xylem(expose)]
struct Bar {
    #[xylem(args(new = true))]
    id:    Id<Schema, Bar>,
    other: Option<Id<Schema, Bar>>,
}

impl Identifiable<Schema> for Bar {
    type Scope = Foo;

    fn id(&self) -> Id<Schema, Self> { self.id }
}

#[test]
fn test_scoped_id() {
    let mut context = DefaultContext::default();

    let first = Foo::convert(
        FooXylem {
            id:    String::from("first"),
            other: None,
            bar:   vec![
                BarXylem { id: String::from("alpha"), other: None },
                BarXylem { id: String::from("beta"), other: Some(String::from("alpha")) },
            ],
        },
        &mut context,
        NoArgs,
    )
    .unwrap();

    assert_eq!(first.bar[0].id.index(), 0);
    assert_eq!(first.bar[1].id.index(), 1);
    assert!(match first.bar[1].other {
        Some(id) => id.index() == 0,
        None => false,
    });

    let second_err = Foo::convert(
        FooXylem {
            id:    String::from("second"),
            other: None,
            bar:   vec![BarXylem {
                id:    String::from("gamma"),
                other: Some(String::from("alpha")),
            }],
        },
        &mut context,
        NoArgs,
    )
    .unwrap_err();
    assert_eq!(second_err.to_string(), "Unknown ID alpha");
}

#[test]
fn test_cross_scope_id() {
    let mut context = DefaultContext::default();

    let first = Foo::convert(
        FooXylem {
            id:    String::from("first"),
            other: None,
            bar:   vec![
                BarXylem { id: String::from("alpha"), other: None },
                BarXylem { id: String::from("beta"), other: Some(String::from("alpha")) },
            ],
        },
        &mut context,
        NoArgs,
    )
    .unwrap();

    assert_eq!(first.bar[0].id.index(), 0);
    assert_eq!(first.bar[1].id.index(), 1);
    assert!(match first.bar[1].other {
        Some(id) => id.index() == 0,
        None => false,
    });

    let second_err = Foo::convert(
        FooXylem {
            id:    String::from("second"),
            other: None,
            bar:   vec![BarXylem {
                id:    String::from("gamma"),
                other: Some(String::from("alpha")),
            }],
        },
        &mut context,
        NoArgs,
    )
    .unwrap_err();
    assert_eq!(second_err.to_string(), "Unknown ID alpha");
}
