use xylem::{DefaultContext, NoArgs, Processable, Xylem, declare_schema};

declare_schema!(Schema);

impl Xylem<Schema> for u32 {
    type From = Self;
    type Args = NoArgs;

    fn convert_impl(from: Self::From, _context: &mut DefaultContext, _args: &Self::Args) -> Result<Self, anyhow::Error> {
        Ok(from)
    }
}

#[derive(Xylem)]
#[xylem(process)]
#[xylem(expose = FooFrom)]
struct Foo {
    bar: u32,
}

impl Processable<Schema> for Foo {
    fn preprocess(from: &mut Self::From, _context: &mut DefaultContext) -> Result<(), anyhow::Error> {
        from.bar += 1;
        Ok(())
    }

    fn postprocess(&mut self, _context: &mut DefaultContext) -> Result<(), anyhow::Error> {
        assert_eq!(self.bar, 5);
        self.bar += 2;
        Ok(())
    }
}

#[test]
fn test_processable() {
    let mut context = DefaultContext::default();
    let foo_xylem = FooFrom {
        bar: 4,
    };

    let foo = Foo::convert(foo_xylem, &mut context, &NoArgs).unwrap();
    assert_eq!(foo.bar, 7);
}
