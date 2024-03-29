use proc_macro2::TokenStream;
use quote::quote;

use crate::tests::token_stream_equals;
use crate::xylem_impl;

fn test_ok(input: TokenStream, expect_from_decl: TokenStream, expect_xylem_impl: TokenStream) {
    let output = xylem_impl(input).expect("Proc macro returned with compile error");

    assert!(
        token_stream_equals(expect_from_decl.clone(), output.from_decl.clone()),
        "Expected `From` declaration\n{}\n, actual `From` declaration\n{}\n",
        &expect_from_decl,
        &output.from_decl
    );

    assert!(
        token_stream_equals(expect_xylem_impl.clone(), output.xylem_impl.clone()),
        "Expected `Xylem` impl:\n{}\n, actual `Xylem` impl:\n{}\n",
        &expect_xylem_impl,
        &output.xylem_impl
    );
}

#[test]
fn test_named_struct() {
    test_ok(
        quote! {
            struct Foo {
                bar: Bar,
                qux: Qux,
            }
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            struct FooXylem {
                bar: <Bar as ::xylem::Xylem<crate::Schema>>::From,
                qux: <Qux as ::xylem::Xylem<crate::Schema>>::From,
            }
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl ::xylem::Xylem<crate::Schema> for Foo {
                type From = FooXylem;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    let mut __xylem_ret = Self {
                        bar: {
                            type Args = <Bar as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.bar, __xylem_context, &*__XYLEM_ARGS)?
                        },
                        qux: {
                            type Args = <Qux as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.qux, __xylem_context, &*__XYLEM_ARGS)?
                        },
                    };
                    Ok(__xylem_ret)
                }
            }
        },
    );
}

#[test]
fn test_tuple_struct() {
    test_ok(
        quote! {
            struct Foo(Bar, Qux);
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            struct FooXylem(
                <Bar as ::xylem::Xylem<crate::Schema>>::From,
                <Qux as ::xylem::Xylem<crate::Schema>>::From,
            );
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl ::xylem::Xylem<crate::Schema> for Foo {
                type From = FooXylem;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    let mut __xylem_ret = Self (
                        {
                            type Args = <Bar as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.0, __xylem_context, &*__XYLEM_ARGS)?
                        },
                        {
                            type Args = <Qux as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.1, __xylem_context, &*__XYLEM_ARGS)?
                        },
                    );
                    Ok(__xylem_ret)
                }
            }
        },
    );
}

#[test]
fn test_unit_struct() {
    test_ok(
        quote! {
            struct Foo;
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            struct FooXylem;
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl ::xylem::Xylem<crate::Schema> for Foo {
                type From = FooXylem;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    let mut __xylem_ret = Self;
                    Ok(__xylem_ret)
                }
            }
        },
    );
}

#[test]
fn test_generic_named_struct() {
    test_ok(
        quote! {
            struct Foo<T: U, U> where U: Corge<T> {
                bar: Bar<T>,
                qux: Qux<U>,
            }
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            struct FooXylem<T: U, U> where U: Corge<T> {
                bar: <Bar<T> as ::xylem::Xylem<crate::Schema>>::From,
                qux: <Qux<U> as ::xylem::Xylem<crate::Schema>>::From,
            }
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl<T: U, U> ::xylem::Xylem<crate::Schema> for Foo<T, U> {
                type From = FooXylem<T, U>;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    let mut __xylem_ret = Self {
                        bar: {
                            type Args = <Bar<T> as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.bar, __xylem_context, &*__XYLEM_ARGS)?
                        },
                        qux: {
                            type Args = <Qux<U> as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.qux, __xylem_context, &*__XYLEM_ARGS)?
                        },
                    };
                    Ok(__xylem_ret)
                }
            }
        },
    );
}

#[test]
fn test_generic_tuple_struct() {
    test_ok(
        quote! {
            struct Foo<T: U, U>(Bar<T>, Qux<U>) where U: Corge<T>;
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            struct FooXylem<T: U, U>(
                <Bar<T> as ::xylem::Xylem<crate::Schema>>::From,
                <Qux<U> as ::xylem::Xylem<crate::Schema>>::From,
            ) where U: Corge<T>;
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl<T: U, U> ::xylem::Xylem<crate::Schema> for Foo<T, U> {
                type From = FooXylem<T, U>;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    let mut __xylem_ret = Self (
                        {
                            type Args = <Bar<T> as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.0, __xylem_context, &*__XYLEM_ARGS)?
                        },
                        {
                            type Args = <Qux<U> as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.1, __xylem_context, &*__XYLEM_ARGS)?
                        },
                    );
                    Ok(__xylem_ret)
                }
            }
        },
    );
}

#[test]
fn test_enum() {
    test_ok(
        quote! {
            enum Foo {
                Bar,
                Qux(Corge, Quz),
                Grault {
                    waldo: Waldo,
                    fred: Fred,
                }
            }
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            enum FooXylem {
                #[doc = concat!("See [`", stringify!(Foo), "::", stringify!(Bar), "`]")]
                Bar,
                #[doc = concat!("See [`", stringify!(Foo), "::", stringify!(Qux), "`]")]
                Qux(
                    <Corge as ::xylem::Xylem<crate::Schema>>::From,
                    <Quz as ::xylem::Xylem<crate::Schema>>::From,
                ),
                #[doc = concat!("See [`", stringify!(Foo), "::", stringify!(Grault), "`]")]
                Grault {
                    waldo: <Waldo as ::xylem::Xylem<crate::Schema>>::From,
                    fred: <Fred as ::xylem::Xylem<crate::Schema>>::From,
                }
            }
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl ::xylem::Xylem<crate::Schema> for Foo {
                type From = FooXylem;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    let mut __xylem_ret = match __xylem_from {
                        FooXylem::Bar => Self::Bar,
                        FooXylem::Qux(__field0, __field1) => Self::Qux(
                            {
                                type Args = <Corge as ::xylem::Xylem<crate::Schema>>::Args;
                                ::xylem::lazy_static! {
                                    static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                                }
                                ::xylem::Xylem::<crate::Schema>::convert(__field0, __xylem_context, &*__XYLEM_ARGS)?
                            },
                            {
                                type Args = <Quz as ::xylem::Xylem<crate::Schema>>::Args;
                                ::xylem::lazy_static! {
                                    static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                                }
                                ::xylem::Xylem::<crate::Schema>::convert(__field1, __xylem_context, &*__XYLEM_ARGS)?
                            },
                        ),
                        FooXylem::Grault { waldo, fred } => Self::Grault {
                            waldo: {
                                type Args = <Waldo as ::xylem::Xylem<crate::Schema>>::Args;
                                ::xylem::lazy_static! {
                                    static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                                }
                                ::xylem::Xylem::<crate::Schema>::convert(waldo, __xylem_context, &*__XYLEM_ARGS)?
                            },
                            fred: {
                                type Args = <Fred as ::xylem::Xylem<crate::Schema>>::Args;
                                ::xylem::lazy_static! {
                                    static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                                }
                                ::xylem::Xylem::<crate::Schema>::convert(fred, __xylem_context, &*__XYLEM_ARGS)?
                            },
                        },
                    };
                    Ok(__xylem_ret)
                }
            }
        },
    );
}

#[test]
fn test_processable() {
    test_ok(
        quote! {
            #[xylem(process)]
            struct Foo {
                bar: Bar,
                qux: Qux,
            }
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            struct FooXylem {
                bar: <Bar as ::xylem::Xylem<crate::Schema>>::From,
                qux: <Qux as ::xylem::Xylem<crate::Schema>>::From,
            }
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl ::xylem::Xylem<crate::Schema> for Foo {
                type From = FooXylem;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    <Self as ::xylem::Processable<crate::Schema>>::preprocess(&mut __xylem_from, __xylem_context)?;
                    let mut __xylem_ret = Self {
                        bar: {
                            type Args = <Bar as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.bar, __xylem_context, &*__XYLEM_ARGS)?
                        },
                        qux: {
                            type Args = <Qux as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.qux, __xylem_context, &*__XYLEM_ARGS)?
                        },
                    };
                    <Self as ::xylem::Processable<crate::Schema>>::postprocess(&mut __xylem_ret, __xylem_context)?;
                    Ok(__xylem_ret)
                }
            }
        },
    );
}

#[test]
fn test_attrs() {
    test_ok(
        quote! {
            #[xylem(derive(Deserialize), serde(bound = ""))]
            struct Foo {
                bar: Bar,
                #[xylem(serde(default))]
                qux: Qux,
            }
        },
        quote! {
            #[doc = concat!("See [`", stringify!(FooXylem), "`]")]
            #[automatically_derived]
            #[derive(Deserialize)]
            #[serde(bound = "")]
            struct FooXylem {
                bar: <Bar as ::xylem::Xylem<crate::Schema>>::From,
                #[serde(default)]
                qux: <Qux as ::xylem::Xylem<crate::Schema>>::From,
            }
        },
        quote! {
            #[automatically_derived]
            #[allow(clippy::needless_update)]
            impl ::xylem::Xylem<crate::Schema> for Foo {
                type From = FooXylem;
                type Args = ::xylem::NoArgs;
                fn convert_impl(
                    mut __xylem_from: Self::From,
                    __xylem_context: &mut <crate::Schema as ::xylem::Schema>::Context,
                    _: &Self::Args,
                ) -> Result<Self, <crate::Schema as ::xylem::Schema>::Error> {
                    let mut __xylem_ret = Self {
                        bar: {
                            type Args = <Bar as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.bar, __xylem_context, &*__XYLEM_ARGS)?
                        },
                        qux: {
                            type Args = <Qux as ::xylem::Xylem<crate::Schema>>::Args;
                            ::xylem::lazy_static! {
                                static ref __XYLEM_ARGS: Args = Args { ..::std::default::Default::default() };
                            }
                            ::xylem::Xylem::<crate::Schema>::convert(__xylem_from.qux, __xylem_context, &*__XYLEM_ARGS)?
                        },
                    };
                    Ok(__xylem_ret)
                }
            }
        },
    );
}
