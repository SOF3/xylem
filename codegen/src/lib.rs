use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Result};

mod tests;

#[proc_macro_derive(Xylem, attributes(xylem))]
pub fn xylem(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match xylem_impl(ts.into()) {
        Ok(output) => output.output(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn xylem_impl(ts: TokenStream) -> Result<Output> {
    let input = syn::parse2::<syn::DeriveInput>(ts)?;
    let input_ident = &input.ident;

    let mut from_ident = None;
    let mut schema = Box::new(
        syn::parse2::<syn::Type>(quote!(crate::Schema))
            .expect("Failed to parse literal token stream"),
    );
    let mut expose_from_type = false;
    let mut input_serde = Vec::new();
    let mut derive_list = Vec::new();

    let mut processable = false;

    for attr in &input.attrs {
        if attr.path.is_ident("xylem") {
            let attr_list: Punctuated<InputAttr, syn::Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;
            for attr in attr_list {
                match attr {
                    InputAttr::Expose(ident) => {
                        expose_from_type = true;
                        from_ident = Some(ident);
                    }
                    InputAttr::Schema(new_schema) => schema = new_schema,
                    InputAttr::Derive(macros) => derive_list.extend(macros),
                    InputAttr::Serde(ts) => input_serde.push(quote!(#[serde(#ts)])),
                    InputAttr::Process => {
                        processable = true;
                    }
                }
            }
        }
    }

    let preprocess = processable.then(|| quote!(<Self as ::xylem::Processable<#schema>>::preprocess(&mut __xylem_from, __xylem_context)?;));
    let postprocess = processable.then(|| quote!(<Self as ::xylem::Processable<#schema>>::postprocess(&mut __xylem_ret, __xylem_context)?;));

    let from_ident = from_ident.unwrap_or_else(|| format_ident!("{}Xylem", &input.ident));

    let vis = if expose_from_type {
        let vis = &input.vis;
        quote!(#vis)
    } else {
        quote!()
    };

    let (generics_decl, _generics_decl_bare, generics_usage, _generics_usage_bare) =
        if input.generics.params.is_empty() {
            (quote!(), quote!(), quote!(), quote!())
        } else {
            let decl: Vec<_> = input.generics.params.iter().collect();
            let usage: Vec<_> = input
                .generics
                .params
                .iter()
                .map(|param| match param {
                    syn::GenericParam::Type(syn::TypeParam { ident, .. }) => quote!(#ident),
                    syn::GenericParam::Lifetime(syn::LifetimeDef { lifetime, .. }) => {
                        quote!(#lifetime)
                    }
                    syn::GenericParam::Const(syn::ConstParam { ident, .. }) => quote!(#ident),
                })
                .collect();
            (quote!(<#(#decl),*>), quote!(#(#decl),*), quote!(<#(#usage),*>), quote!(#(#usage),*))
        };
    let generics_where = &input.generics.where_clause;

    let derive = (!derive_list.is_empty()).then(|| {
        quote! {
            #[derive(#(#derive_list),*)]
        }
    });

    let prefix = quote! {
        #[doc = concat!("See [`", stringify!(#from_ident), "`]")]
        #[automatically_derived]
        #derive
        #(#input_serde)*
    };

    let (from_decl, convert_expr) = match &input.data {
        syn::Data::Struct(data) => {
            let mut field_froms = Vec::new();
            let mut field_convs = Vec::new();

            for (field_ord, field) in data.fields.iter().enumerate() {
                let (from, conv) = process_field(
                    field,
                    match &field.ident {
                        Some(field_ident) => quote!(__xylem_from.#field_ident),
                        None => {
                            let field_ord = proc_macro2::Literal::usize_unsuffixed(field_ord);
                            quote!(__xylem_from.#field_ord)
                        }
                    },
                    &schema,
                )?;
                if let Some(from) = from {
                    field_froms.push(from);
                }
                field_convs.push(conv);
            }

            let field_froms_attrs: Vec<_> = field_froms.iter().map(|ff| &ff.attrs).collect();
            let field_froms_ident: Vec<_> = field_froms.iter().map(|ff| &ff.ident).collect();
            let field_froms_ty: Vec<_> = field_froms.iter().map(|ff| &ff.ty).collect();
            let field_convs_ident: Vec<_> = field_convs.iter().map(|fc| &fc.ident).collect();
            let field_convs_expr: Vec<_> = field_convs.iter().map(|fc| &fc.expr).collect();

            match &data.fields {
                syn::Fields::Named(_) => (
                    quote! {
                        #prefix
                        #vis struct #from_ident #generics_decl #generics_where {
                            #(
                                #field_froms_attrs
                                #field_froms_ident: #field_froms_ty,
                            )*
                        }
                    },
                    quote! {
                        Self {
                            #(
                                #field_convs_ident: #field_convs_expr,
                            )*
                        }
                    },
                ),
                syn::Fields::Unnamed(_) => (
                    quote! {
                        #prefix
                        #vis struct #from_ident #generics_decl (
                            #(#field_froms_attrs #field_froms_ty,)*
                        ) #generics_where;
                    },
                    quote! {
                        Self (
                            #(#field_convs_expr,)*
                        )
                    },
                ),
                syn::Fields::Unit => (
                    quote! {
                        #prefix
                        #vis struct #from_ident;
                    },
                    quote! {
                        Self
                    },
                ),
            }
        }
        syn::Data::Enum(data) => {
            let mut variant_froms = Vec::new();
            let mut variant_matches = Vec::new();

            for variant in &data.variants {
                let mut field_froms = Vec::new();
                let mut field_convs = Vec::new();

                for (field_ord, field) in variant.fields.iter().enumerate() {
                    let (from, conv) = process_field(
                        field,
                        match &field.ident {
                            Some(ident) => quote!(#ident),
                            None => format_ident!("__field{}", field_ord).to_token_stream(),
                        },
                        &schema,
                    )?;
                    if let Some(from) = from {
                        field_froms.push(from);
                    }
                    field_convs.push(conv);
                }

                let field_froms_attrs: Vec<_> = field_froms.iter().map(|ff| &ff.attrs).collect();
                let field_froms_ident: Vec<_> = field_froms.iter().map(|ff| &ff.ident).collect();
                let field_froms_ty: Vec<_> = field_froms.iter().map(|ff| &ff.ty).collect();

                let variant_from_ident = &variant.ident;
                let variant_from_fields = match &variant.fields {
                    syn::Fields::Named(_) => {
                        quote! {{
                            #(
                                #field_froms_attrs
                                #field_froms_ident: #field_froms_ty,
                            )*
                        }}
                    }
                    syn::Fields::Unnamed(_) => {
                        quote! {(
                            #(#field_froms_attrs #field_froms_ty),*
                        )}
                    }
                    syn::Fields::Unit => quote!(),
                };

                let variant_from = quote! {
                    #variant_from_ident #variant_from_fields
                };
                variant_froms.push(variant_from);

                let variant_from_fields_pat = match &variant.fields {
                    syn::Fields::Named(_) => {
                        quote!({ #(#field_froms_ident),*  })
                    }
                    syn::Fields::Unnamed(_) => {
                        let numbered_fields = (0..variant.fields.len()).map(|field_ord| {
                            format_ident!("__field{}", field_ord).to_token_stream()
                        });
                        quote!((#(#numbered_fields),*))
                    }
                    syn::Fields::Unit => quote!(),
                };

                let variant_to_ident = &variant.ident;

                let field_convs_ident: Vec<_> = field_convs.iter().map(|fc| &fc.ident).collect();
                let field_convs_expr: Vec<_> = field_convs.iter().map(|fc| &fc.expr).collect();
                let variant_to_fields_expr = match &variant.fields {
                    syn::Fields::Named(_) => {
                        quote!({ #(#field_convs_ident: #field_convs_expr),* })
                    }
                    syn::Fields::Unnamed(_) => {
                        quote!((#(#field_convs_expr),*))
                    }
                    syn::Fields::Unit => quote!(),
                };

                let variant_match = quote! {
                    #from_ident::#variant_from_ident #variant_from_fields_pat =>
                        Self::#variant_to_ident #variant_to_fields_expr
                };
                variant_matches.push(variant_match);
            }

            (
                quote! {
                    #prefix
                    #vis enum #from_ident #generics_decl #generics_where {
                        #(#variant_froms),*
                    }
                },
                quote! {
                    match __xylem_from {
                        #(#variant_matches),*
                    }
                },
            )
        }
        syn::Data::Union(data) => {
            return Err(Error::new_spanned(&data.union_token, "Unions are not supported"));
        }
    };

    let xylem_impl = quote! {
        #[automatically_derived]
        impl #generics_decl ::xylem::Xylem<#schema> for #input_ident #generics_usage {
            type From = #from_ident #generics_usage;
            type Args = ::xylem::NoArgs;

            fn convert_impl(
                mut __xylem_from: Self::From,
                __xylem_context: &mut <#schema as ::xylem::Schema>::Context,
                _: &Self::Args,
            ) -> Result<Self, <#schema as ::xylem::Schema>::Error> {
                #preprocess
                let mut __xylem_ret = #convert_expr;
                #postprocess
                Ok(__xylem_ret)
            }
        }
    };
    Ok(Output { from_decl, xylem_impl, expose_from_type })
}

struct Output {
    from_decl:        TokenStream,
    xylem_impl:       TokenStream,
    expose_from_type: bool,
}

impl Output {
    fn output(&self) -> TokenStream {
        let from_decl = &self.from_decl;
        let xylem_impl = &self.xylem_impl;

        let inner = quote! {
            #from_decl
            #xylem_impl
        };

        if self.expose_from_type {
            quote! {
                #inner
            }
        } else {
            quote! {
                const _: () = { #inner };
            }
        }
    }
}

enum InputAttr {
    /// Exposes the `From` type in the same namespace and visibility as the derive input
    /// using the specified identifier as the type name.
    Expose(syn::Ident),
    /// Specifies the schema that the conversion is defined for.
    /// The default value is `crate::Schema`.
    Schema(Box<syn::Type>),
    /// Adds a serde attribute to the `From` type.
    Serde(TokenStream),
    /// Adds a derive macro to the `From` type.
    Derive(Punctuated<syn::Path, syn::Token![,]>),
    /// Call [`Processable`].
    Process,
}

impl Parse for InputAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident == "expose" {
            let _: syn::Token![=] = input.parse()?;
            Ok(Self::Expose(input.parse()?))
        } else if ident == "schema" {
            let _: syn::Token![=] = input.parse()?;
            let schema: syn::Type = input.parse()?;
            Ok(Self::Schema(Box::new(schema)))
        } else if ident == "serde" {
            let inner;
            syn::parenthesized!(inner in input);
            Ok(Self::Serde(inner.parse()?))
        } else if ident == "derive" {
            let inner;
            syn::parenthesized!(inner in input);
            Ok(Self::Derive(Punctuated::parse_terminated(&inner)?))
        } else if ident == "process" {
            Ok(Self::Process)
        } else {
            Err(Error::new_spanned(ident, "Unsupported attribute"))
        }
    }
}

enum FieldAttr {
    /// Adds a serde attribute to the field.
    Serde(TokenStream),
    /// Preserve the field type, without performing any conversion logic.
    Preserve(Span),
    /// Use the specified function to convert the field.
    ///
    /// # Example
    /// ```ignore
    /// #[xylem(transform = path(Type))]
    /// foo: Bar,
    /// ```
    ///
    /// This expects a function accessible at `path`
    /// with the signature `fn(Type) -> Result<Bar, S::Error>`.
    /// For example, `#[xylem(transform = Ok(Bar))]` is equivalent to `#[xylem(preserve)]`.
    ///
    /// # Comparison with [`FieldAttr::Default`]
    /// `transform` differs from `default` in that
    /// `transform` generates a field in the `From` type and passes it to the function,
    /// while `default` does not generate a field in the `From` type
    /// and the argument is a freeform expression.
    Transform(syn::Path, syn::Type),
    /// Similar to [`FieldAttr::Transform`], but also accepts the `context` parameter.
    ///
    /// The signature is `fn(Type, &mut S::Context) -> Result<Bar, S::Error>`.
    TransformWithContext(syn::Path, syn::Type),
    /// Use the specified expression to generate the field value.
    /// The field does not appear in the `From` type.
    Default(syn::Expr),
    /// Pass arguments to the field type.
    Args(Span, Punctuated<ArgDef, syn::Token![,]>),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident == "serde" {
            let inner;
            syn::parenthesized!(inner in input);
            Ok(Self::Serde(inner.parse()?))
        } else if ident == "preserve" {
            Ok(Self::Preserve(ident.span()))
        } else if ident == "transform" {
            let _: syn::Token![=] = input.parse()?;
            let path: syn::Path = input.parse()?;
            let inner;
            syn::parenthesized!(inner in input);
            let ty: syn::Type = inner.parse()?;
            Ok(Self::Transform(path, ty))
        } else if ident == "transform_with_context" {
            let _: syn::Token![=] = input.parse()?;
            let path: syn::Path = input.parse()?;
            let inner;
            syn::parenthesized!(inner in input);
            let ty: syn::Type = inner.parse()?;
            Ok(Self::TransformWithContext(path, ty))
        } else if ident == "default" {
            let _: syn::Token![=] = input.parse()?;
            let expr: syn::Expr = input.parse()?;
            Ok(Self::Default(expr))
        } else if ident == "args" {
            let inner;
            syn::parenthesized!(inner in input);
            Ok(Self::Args(ident.span(), Punctuated::parse_terminated(&inner)?))
        } else {
            Err(Error::new_spanned(ident, "Unsupported attribute"))
        }
    }
}

struct ArgDef {
    name: syn::Ident,
    expr: syn::Expr,
}

impl Parse for ArgDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: syn::Ident = input.parse()?;
        let _: syn::Token![=] = input.parse()?;
        let expr: syn::Expr = input.parse()?;
        Ok(Self { name, expr })
    }
}

fn process_field(
    field: &syn::Field,
    from_expr: TokenStream,
    schema: &syn::Type,
) -> Result<(Option<FieldFrom>, FieldConv)> {
    enum Mode {
        Standard(Vec<ArgDef>),
        Default(TokenStream),
        Transform { ts: TokenStream, ty: Box<syn::Type>, context: bool },
    }

    let mut mode = Mode::Standard(Vec::new());

    let mut from_attrs = TokenStream::new();

    for attr in &field.attrs {
        if attr.path.is_ident("xylem") {
            let attrs: Punctuated<FieldAttr, syn::Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;
            for attr in attrs {
                match attr {
                    FieldAttr::Serde(ts) => {
                        from_attrs.extend(quote!(#[serde(#ts)]));
                    }
                    FieldAttr::Preserve(span) => {
                        if !matches!(mode, Mode::Standard(_)) {
                            return Err(Error::new(
                                span,
                                "Only one of `preserve`, `transform` or `default` can be used.",
                            ));
                        }
                        mode = Mode::Transform {
                            ts:      quote!(Ok),
                            ty:      Box::new(field.ty.clone()),
                            context: false,
                        };
                    }
                    FieldAttr::Transform(path, ty) => {
                        if !matches!(mode, Mode::Standard(_)) {
                            return Err(Error::new_spanned(
                                path,
                                "Only one of `preserve`, `transform` or `default` can be used.",
                            ));
                        }
                        mode = Mode::Transform {
                            ts:      quote!(#path),
                            ty:      Box::new(ty),
                            context: false,
                        };
                    }
                    FieldAttr::TransformWithContext(path, ty) => {
                        if !matches!(mode, Mode::Standard(_)) {
                            return Err(Error::new_spanned(
                                path,
                                "Only one of `preserve`, `transform` or `default` can be used.",
                            ));
                        }
                        mode = Mode::Transform {
                            ts:      quote!(#path),
                            ty:      Box::new(ty),
                            context: true,
                        };
                    }
                    FieldAttr::Default(expr) => {
                        if !matches!(mode, Mode::Standard(_)) {
                            return Err(Error::new_spanned(
                                expr,
                                "Only one of `preserve`, `transform` or `default` can be used.",
                            ));
                        }
                        mode = Mode::Default(quote!(#expr));
                    }
                    FieldAttr::Args(span, args) => match &mut mode {
                        Mode::Standard(arg_defs) => {
                            arg_defs.extend(args.into_iter());
                        }
                        _ => {
                            return Err(Error::new(
                                span,
                                "Cannot use `args` if `preserve`, `transform` or `default` is \
                                 used.",
                            ))
                        }
                    },
                }
            }
        }
    }

    Ok(match mode {
        Mode::Standard(arg_defs) => (
            Some(FieldFrom {
                attrs: from_attrs,
                ident: field.ident.clone(),
                ty:    {
                    let ty = &field.ty;
                    quote!(<#ty as ::xylem::Xylem<#schema>>::From)
                },
            }),
            FieldConv {
                ident: field.ident.clone(),
                expr:  {
                    let ty = &field.ty;
                    let arg_names = arg_defs.iter().map(|def| &def.name);
                    let arg_exprs = arg_defs.iter().map(|def| &def.expr);

                    quote! {{
                        type Args = <#ty as ::xylem::Xylem<#schema>>::Args;
                        ::xylem::lazy_static! {
                            static ref __XYLEM_ARGS: Args = Args {
                                #(#arg_names: #arg_exprs,)*
                                ..::std::default::Default::default()
                            };
                        }
                        ::xylem::Xylem::<#schema>::convert(
                            #from_expr,
                            __xylem_context,
                            &*__XYLEM_ARGS,
                        )?
                    }}
                },
            },
        ),
        Mode::Default(expr) => (None, FieldConv { ident: field.ident.clone(), expr }),
        Mode::Transform { ts, ty, context } => {
            let context = context.then(|| quote!(__xylem_context));
            (
                Some(FieldFrom {
                    attrs: from_attrs,
                    ident: field.ident.clone(),
                    ty:    quote!(#ty),
                }),
                FieldConv {
                    ident: field.ident.clone(),
                    expr:  quote! {
                        #ts(#from_expr, #context)?
                    },
                },
            )
        }
    })
}

#[derive(Debug)]
struct FieldFrom {
    /// The attributes of the field in the `From` type.
    attrs: TokenStream,
    /// The name of the field in the `From` type.
    ident: Option<syn::Ident>,
    /// The type of the field in the `From` type.
    ty:    TokenStream,
}

#[derive(Debug)]
struct FieldConv {
    /// The name of the field in the `Self` type.
    ident: Option<syn::Ident>,
    /// The expression of the field in the constructor.
    expr:  TokenStream,
}
