use std::any::TypeId;

use xylem::{DefaultContext, Id, Identifiable, NoArgs, SchemaExt, Xylem};

enum Schema {}

impl xylem::Schema for Schema {
    type Context = DefaultContext;

    type Error = anyhow::Error;
}

impl SchemaExt for Schema {}

#[derive(Xylem)]
#[xylem(expose)]
struct Foo {
    #[xylem(args(import = vec![TypeId::of::<Qux>()]))]
    bar: Id<Schema, Bar>,
    qux: Id<Schema, Qux>,
}

#[derive(Xylem)]
#[xylem(expose)]
struct Bar {
    #[xylem(args(new = true))]
    id:  Id<Schema, Bar>,
    #[allow(dead_code)] // it's only used to provide type context.
    qux: Vec<Qux>,
}

impl Identifiable<Schema> for Bar {
    type Scope = ();
    fn id(&self) -> Id<Schema, Bar> { self.id }
}

#[derive(Xylem)]
#[xylem(expose)]
struct Qux {
    #[xylem(args(new = true, track = true))]
    id: Id<Schema, Qux>,
}

impl Identifiable<Schema> for Qux {
    type Scope = Bar;
    fn id(&self) -> Id<Schema, Qux> { self.id }
}

#[test]
fn test_cross_ref() {
    let mut context = DefaultContext::default();

    Bar::convert(
        BarXylem {
            id:  String::from("one"),
            qux: vec![QuxXylem { id: String::from("two") }, QuxXylem { id: String::from("three") }],
        },
        &mut context,
        &NoArgs,
    )
    .unwrap();

    let foo = Foo::convert(
        FooXylem { bar: String::from("one"), qux: String::from("three") },
        &mut context,
        &NoArgs,
    )
    .unwrap();
    assert_eq!(foo.bar.index(), 0);
    assert_eq!(foo.qux.index(), 1);
}
