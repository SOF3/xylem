use xylem::{declare_schema, DefaultContext, Id, Identifiable, NoArgs, SchemaExt, Xylem};

declare_schema!(MySchema: SchemaExt);

// Test for cross-referencing IDs

#[derive(Debug, Xylem)]
#[xylem(schema = MySchema, expose = FooFrom)]
struct Foo {
    #[xylem(args(new = true))]
    id:    Id<MySchema, Foo>,
    other: Option<Id<MySchema, Foo>>,
    bar:   Vec<Bar>,
}

impl Identifiable<MySchema> for Foo {
    type Scope = ();

    fn id(&self) -> Id<MySchema, Self> { self.id }
}

#[test]
fn test_ref() {
    let mut context = DefaultContext::default();

    let first = Foo::convert(
        FooFrom { id: String::from("first"), other: None, bar: Vec::new() },
        &mut context,
        &NoArgs,
    )
    .unwrap();

    assert_eq!(first.id.index(), 0);
    assert!(first.other.is_none());

    let second = Foo::convert(
        FooFrom { id: String::from("second"), other: None, bar: Vec::new() },
        &mut context,
        &NoArgs,
    )
    .unwrap();

    assert_eq!(second.id.index(), 1);
    assert!(second.other.is_none());

    let third = Foo::convert(
        FooFrom {
            id:    String::from("third"),
            other: Some(String::from("first")),
            bar:   Vec::new(),
        },
        &mut context,
        &NoArgs,
    )
    .unwrap();

    assert_eq!(third.id.index(), 2);
    assert!(match third.other {
        Some(id) => id.index() == 0,
        None => false,
    });
}

// Test for scoped IDs

#[derive(Debug, Xylem)]
#[xylem(schema = MySchema, expose = BarFrom)]
struct Bar {
    #[xylem(args(new = true))]
    id:    Id<MySchema, Bar>,
    other: Option<Id<MySchema, Bar>>,
}

impl Identifiable<MySchema> for Bar {
    type Scope = Foo;

    fn id(&self) -> Id<MySchema, Self> { self.id }
}

#[test]
fn test_scoped_id() {
    let mut context = DefaultContext::default();

    let first = Foo::convert(
        FooFrom {
            id:    String::from("first"),
            other: None,
            bar:   vec![
                BarFrom { id: String::from("alpha"), other: None },
                BarFrom { id: String::from("beta"), other: Some(String::from("alpha")) },
            ],
        },
        &mut context,
        &NoArgs,
    )
    .unwrap();

    assert_eq!(first.bar[0].id.index(), 0);
    assert_eq!(first.bar[1].id.index(), 1);
    assert!(match first.bar[1].other {
        Some(id) => id.index() == 0,
        None => false,
    });

    let second_err = Foo::convert(
        FooFrom {
            id:    String::from("second"),
            other: None,
            bar:   vec![BarFrom {
                id:    String::from("gamma"),
                other: Some(String::from("alpha")),
            }],
        },
        &mut context,
        &NoArgs,
    )
    .unwrap_err();
    assert_eq!(second_err.to_string(), "Unknown ID alpha");
}
