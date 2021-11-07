use anyhow::Context as _;
use xylem::{DefaultContext, NoArgs, Xylem};

enum Schema {}

impl xylem::Schema for Schema {
    type Context = xylem::DefaultContext;

    type Error = anyhow::Error;
}

// Implement `Vec<T>` in a new way
// to test that we do not have E0119 with [`VecSchemaExt`].
impl Xylem<Schema> for Vec<i32> {
    type From = Vec<String>; // i32: Xylem<From = String> is not true.
    type Args = NoArgs;

    fn convert_impl(
        from: Self::From,
        _: &mut DefaultContext,
        _: Self::Args,
    ) -> Result<Self, anyhow::Error> {
        from.into_iter().map(|item| item.parse().context("Parse error")).collect()
    }
}

#[test]
fn test_cross_ref() {}
