use std::any::TypeId;

use xylem::{declare_schema, DefaultContext, Id, Identifiable, NoArgs, SchemaExt, Xylem};

declare_schema!(Schema: SchemaExt);

#[derive(Xylem)]
#[xylem(expose = FooFrom)]
struct Foo {
    #[xylem(args(import = vec![TypeId::of::<Qux>()]))]
    bar: Id<Schema, Bar>,
    qux: Id<Schema, Qux>,
}

#[derive(Xylem)]
#[xylem(expose = BarFrom)]
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
#[xylem(expose = QuxFrom)]
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
        BarFrom {
            id:  String::from("one"),
            qux: vec![QuxFrom { id: String::from("two") }, QuxFrom { id: String::from("three") }],
        },
        &mut context,
        &NoArgs,
    )
    .unwrap();

    let foo = Foo::convert(
        FooFrom { bar: String::from("one"), qux: String::from("three") },
        &mut context,
        &NoArgs,
    )
    .unwrap();
    assert_eq!(foo.bar.index(), 0);
    assert_eq!(foo.qux.index(), 1);
}
