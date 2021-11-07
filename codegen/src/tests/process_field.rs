use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::tests::token_stream_equals;
use crate::{process_field, FieldConv, FieldFrom};

fn test_process_field(fields: TokenStream, expects: &[(Option<FieldFrom>, FieldConv)]) {
    let full = quote! {
        struct Test #fields
    };
    let data = syn::parse2::<syn::ItemStruct>(full).expect("Invalid test case");

    for (field, (expect_from, expect_conv)) in data.fields.iter().zip(expects.iter()) {
        let (actual_from, actual_conv) = process_field(
            field,
            quote!(_from_placeholder_),
            &syn::parse2::<syn::Type>(quote!(::_placeholder_::_Schema_))
                .expect("Cannot parse literal token stream"),
        )
        .expect("Invalid test case");

        match (expect_from, &actual_from) {
            (None, None) => {}
            (Some(expect), Some(actual)) => {
                assert!(
                    token_stream_equals(expect.attrs.clone(), actual.attrs.clone()),
                    "Expected FieldFrom.attrs =\n{}\n, actual FieldFrom.attrs =\n{}\n",
                    &expect.attrs,
                    &actual.attrs,
                );

                match (&expect.ident, &actual.ident) {
                    (None, None) => {}
                    (Some(expect), Some(actual)) => {
                        assert_eq!(expect.to_string(), actual.to_string());
                    }
                    _ => panic!(
                        "Expected FieldFrom.ident = {:?}, actual FieldFrom.ident = {:?}",
                        expect, actual
                    ),
                }

                assert!(
                    token_stream_equals(expect.ty.clone(), actual.ty.clone()),
                    "Expected FieldFrom.ty =\n{}\n, actual FieldFrom.ty =\n{}\n",
                    &expect.ty,
                    &actual.ty,
                );
            }
            _ => panic!("Expected FieldFrom = {:?}, got {:?}", expect_from, actual_from),
        }

        match (&expect_conv.ident, &actual_conv.ident) {
            (None, None) => {}
            (Some(expect), Some(actual)) => {
                assert_eq!(expect.to_string(), actual.to_string());
            }
            (expect, actual) => panic!(
                "Expected FieldConv.ident = {:?}, actual FieldConv.ident = {:?}",
                expect, actual,
            ),
        }

        assert!(
            token_stream_equals(expect_conv.expr.clone(), actual_conv.expr.clone()),
            "Expected FieldConv.expr =\n{}\n, actual FieldConv.expr =\n{}\n",
            &expect_conv.expr,
            &actual_conv.expr
        );
    }
}

#[test]
fn test_field_standard_named() {
    test_process_field(
        quote!({
            foo: Bar,
        }),
        &[(
            Some(FieldFrom {
                attrs: quote! {},
                ident: Some(Ident::new("foo", Span::call_site())),
                ty:    quote!(<Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::From),
            }),
            FieldConv {
                ident: Some(Ident::new("foo", Span::call_site())),
                expr:  quote! {{
                    type Args = <Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::Args;
                    ::xylem::Xylem::<::_placeholder_::_Schema_>::convert(_from_placeholder_, __xylem_context, Args { ..::std::default::Default::default() })?
                }},
            },
        )],
    );
}

#[test]
fn test_field_standard_unnamed() {
    test_process_field(
        quote!((Bar);),
        &[(
            Some(FieldFrom {
                attrs: quote! {},
                ident: None,
                ty:    quote!(<Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::From),
            }),
            FieldConv {
                ident: None,
                expr:  quote! {{
                    type Args = <Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::Args;
                    ::xylem::Xylem::<::_placeholder_::_Schema_>::convert(_from_placeholder_, __xylem_context, Args { ..::std::default::Default::default() })?
                }},
            },
        )],
    );
}

#[test]
fn test_field_serde() {
    test_process_field(
        quote!({
            #[xylem(serde(tagged))]
            foo: Bar,
        }),
        &[(
            Some(FieldFrom {
                attrs: quote! {
                    #[serde(tagged)]
                },
                ident: Some(Ident::new("foo", Span::call_site())),
                ty:    quote!(<Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::From),
            }),
            FieldConv {
                ident: Some(Ident::new("foo", Span::call_site())),
                expr:  quote! {{
                    type Args = <Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::Args;
                    ::xylem::Xylem::<::_placeholder_::_Schema_>::convert(_from_placeholder_, __xylem_context, Args { ..::std::default::Default::default() })?
                }},
            },
        )],
    );
}

#[test]
fn test_field_preserve() {
    test_process_field(
        quote!({
            #[xylem(preserve)]
            foo: Bar,
        }),
        &[(
            Some(FieldFrom {
                attrs: quote! {},
                ident: Some(Ident::new("foo", Span::call_site())),
                ty:    quote!(Bar),
            }),
            FieldConv {
                ident: Some(Ident::new("foo", Span::call_site())),
                expr:  quote! {
                    Ok(_from_placeholder_)?
                },
            },
        )],
    );
}

#[test]
fn test_field_transform() {
    test_process_field(
        quote!({
            #[xylem(transform = qux(Corge))]
            foo: Bar,
        }),
        &[(
            Some(FieldFrom {
                attrs: quote! {},
                ident: Some(Ident::new("foo", Span::call_site())),
                ty:    quote!(Corge),
            }),
            FieldConv {
                ident: Some(Ident::new("foo", Span::call_site())),
                expr:  quote! {
                    qux(_from_placeholder_)?
                },
            },
        )],
    );
}

#[test]
fn test_field_transform_context() {
    test_process_field(
        quote!({
            #[xylem(transform_with_context = qux(Corge))]
            foo: Bar,
        }),
        &[(
            Some(FieldFrom {
                attrs: quote! {},
                ident: Some(Ident::new("foo", Span::call_site())),
                ty:    quote!(Corge),
            }),
            FieldConv {
                ident: Some(Ident::new("foo", Span::call_site())),
                expr:  quote! {
                    qux(_from_placeholder_, __xylem_context)?
                },
            },
        )],
    );
}

#[test]
fn test_field_default() {
    test_process_field(
        quote!({
            #[xylem(default = qux())]
            foo: Bar,
        }),
        &[(
            None,
            FieldConv {
                ident: Some(Ident::new("foo", Span::call_site())),
                expr:  quote! {
                    qux()
                },
            },
        )],
    );
}

#[test]
fn test_field_args() {
    test_process_field(
        quote!({
            #[xylem(args(foo = bar, qux = corge(1, "waldo")))]
            foo: Bar,
        }),
        &[(
            Some(FieldFrom {
                attrs: quote! {},
                ident: Some(Ident::new("foo", Span::call_site())),
                ty:    quote!(<Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::From),
            }),
            FieldConv {
                ident: Some(Ident::new("foo", Span::call_site())),
                expr:  quote! {{
                    type Args = <Bar as ::xylem::Xylem<::_placeholder_::_Schema_>>::Args;
                    ::xylem::Xylem::<::_placeholder_::_Schema_>::convert(
                        _from_placeholder_,
                        __xylem_context,
                        Args {
                            foo: bar,
                            qux: corge(1, "waldo"),
                            ..::std::default::Default::default(),
                        },
                    )?
                }},
            },
        )],
    );
}
