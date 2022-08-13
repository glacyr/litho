use darling::util::Flag;
use darling::{FromAttributes, FromMeta};
use inflections::Inflect;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, AttributeArgs, FnArg, Ident, ImplItem, ImplItemMethod, Item, Lit,
    LitStr, Meta, Pat, PatType, ReturnType, Type,
};

mod derive;

#[derive(Debug, FromMeta)]
struct KonoImpl {
    rename: Option<LitStr>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(kono))]
struct Kono {
    mutation: Flag,
    rename: Option<LitStr>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(kono))]
struct InputAttrs {
    flatten: Flag,
}

#[derive(PartialEq, Eq)]
enum FieldTy {
    Mutation,
    Field,
    Query,
}

struct Input {
    attrs: InputAttrs,
    pat_type: PatType,
}

struct Field {
    ident: Ident,
    name: LitStr,
    has_environment: bool,
    doc: Option<Lit>,
    method: ImplItemMethod,
    ty: FieldTy,
    inputs: Vec<Input>,
    output: ReturnType,
}

fn assoc_type(item: &syn::ItemImpl, name: &str, default: fn() -> TokenStream) -> TokenStream {
    item.items
        .iter()
        .find(|item| match item {
            ImplItem::Type(ty) => ty.ident == name,
            _ => false,
        })
        .to_owned()
        .map(|ty| ty.into_token_stream())
        .unwrap_or(default())
}

fn kono_extract_comment(attrs: &[Attribute]) -> Option<Lit> {
    attrs
        .iter()
        .find_map(|attr| match attr.path.get_ident()?.to_string().as_str() {
            "doc" => match attr.parse_meta().ok()? {
                Meta::NameValue(meta) => Some(meta.lit),
                _ => None,
            },
            _ => None,
        })
}

fn kono_impl_method(method: ImplItemMethod) -> Field {
    let kono = Kono::from_attributes(&method.attrs).unwrap();
    let doc = kono_extract_comment(&method.attrs);

    let mut method = method;
    method.attrs = method
        .attrs
        .into_iter()
        .filter(|attr| !match attr.path.get_ident() {
            Some(ident) => ident.to_string() == "kono",
            _ => false,
        })
        .collect::<Vec<_>>();

    let ident = method.sig.ident.to_owned();
    let name = match kono.rename {
        Some(rename) => LitStr::new(rename.value().trim_start_matches("r#"), rename.span()),
        None => LitStr::new(
            &method
                .sig
                .ident
                .to_string()
                .trim_start_matches("r#")
                .to_camel_case(),
            method.sig.ident.span(),
        ),
    };

    let has_receiver = method
        .sig
        .inputs
        .iter()
        .find(|input| matches!(input, FnArg::Receiver(_)))
        .is_some();

    let has_environment = method
        .sig
        .inputs
        .iter()
        .find(|input| match input {
            FnArg::Typed(pat) => match &*pat.pat {
                Pat::Ident(ident) if ident.ident == "environment" => true,
                _ => false,
            },
            _ => false,
        })
        .is_some();

    let ty = if has_receiver {
        FieldTy::Field
    } else if kono.mutation.is_present() {
        FieldTy::Mutation
    } else {
        FieldTy::Query
    };

    let output = method.sig.output.to_owned();
    let inputs = method
        .sig
        .inputs
        .iter()
        .flat_map(|input| match input {
            FnArg::Typed(pat) => match &*pat.pat {
                Pat::Ident(ident) if ident.ident != "environment" => Some(pat),
                _ => None,
            },
            _ => None,
        })
        .cloned()
        .map(|pat_type| Input {
            attrs: InputAttrs::from_attributes(&pat_type.attrs).unwrap(),
            pat_type,
        })
        .collect();

    for input in method.sig.inputs.iter_mut() {
        match input {
            FnArg::Typed(pat) => {
                pat.attrs = std::mem::take(&mut pat.attrs)
                    .into_iter()
                    .filter(|attr| !match attr.path.get_ident() {
                        Some(ident) => ident.to_string() == "kono",
                        _ => false,
                    })
                    .collect::<Vec<_>>();
            }
            _ => {}
        }
    }

    Field {
        ident,
        name,
        has_environment,
        doc,
        output,
        inputs,
        method,
        ty,
    }
}

fn schema<'a>(fields: impl Iterator<Item = &'a Field>) -> Vec<proc_macro2::TokenStream> {
    fields
        .map(|field| {
            let name = &field.name;
            let description = match &field.doc {
                Some(description) => quote! { .description(#description.trim()) },
                _ => quote! { },
            };
            let ty = match &field.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => quote! { #ty },
            };

            let inputs = field.inputs.iter().map(|input|{
                let name = match &*input.pat_type.pat {
                    Pat::Ident(pat) => pat.ident.to_string(),
                    _ => unreachable!(),
                };

                let ty = &input.pat_type.ty;

                match input.attrs.flatten.is_present() {
                    true => quote_spanned! { ty.span() =>
                        .arguments(<#ty as kono::aspect::ArgumentType<_>>::schema(_environment))
                    },
                    false => quote_spanned! { ty.span() =>
                        .argument(kono::schema::InputValue::new(#name, <#ty as kono::aspect::InputType<_>>::ty(_environment)))
                    }
                }
            });

            quote_spanned! { ty.span() =>
                kono::schema::Field::new(Some(#name), <#ty as kono::aspect::OutputType<_>>::ty(_environment))
                    #description
                    #(#inputs)*,
            }
        })
        .collect::<Vec<_>>()
}

fn handlers<'a, F>(fields: impl Iterator<Item = &'a Field>, f: F) -> Vec<proc_macro2::TokenStream>
where
    F: Fn(proc_macro2::TokenStream, proc_macro2::TokenStream) -> proc_macro2::TokenStream,
{
    fields
        .map(|field| {
            let name = &field.name;
            let ident = field.ident.to_owned();

            let mut args = vec![];

            if field.has_environment {
                args.push(quote! { _environment });
            }

            for input in field.inputs.iter() {
                let name = match &*input.pat_type.pat {
                    Pat::Ident(pat) => pat.ident.to_string().to_camel_case(),
                    _ => unreachable!(),
                };

                let ty = &input.pat_type.ty;
                args.push(match input.attrs.flatten.is_present() {
                    true => quote_spanned! { ty.span() =>
                        <#ty as kono::aspect::ArgumentType::<_>>::from_args(&args, _environment)?
                    },
                    false => quote_spanned! { ty.span() =>
                        <#ty as kono::aspect::InputType::<_>>::from_value_option(args.get(#name).cloned(), _environment)?
                    },
            });
            }

            let im = f(quote! { #ident }, quote! { #(#args),* });

            quote! {
                #name => #im.into_intermediate(_environment),
            }
        })
        .collect::<Vec<_>>()
}

fn kono_impl(kono: KonoImpl, item: syn::Item) -> Result<proc_macro2::TokenStream, String> {
    let item = match item {
        Item::Impl(item) => item,
        _ => return Err("`kono` only supports implementations.".to_owned()),
    };

    let generics = item.generics.clone();
    let self_ty = item.self_ty.clone();
    let name = kono.rename.unwrap_or(match &*self_ty {
        Type::Path(ty) => {
            let ident = &ty.path.segments.last().unwrap().ident;
            LitStr::new(&ident.to_string(), ident.span())
        }
        _ => todo!(),
    });
    let where_clause = item.generics.where_clause.clone();

    let context = assoc_type(&item, "Context", || quote! { type Context = (); });
    let environment = assoc_type(&item, "Environment", || quote! { type Environment = (); });
    let error = assoc_type(
        &item,
        "Error",
        || quote! { type Error = kono::aspect::Error; },
    );

    let (schema_generics, schema_env) = match item.items.iter().find_map(|item| match item {
        ImplItem::Type(ty) if ty.ident == "Environment" => Some(ty),
        _ => None,
    }) {
        Some(item) => (quote! { #generics }, {
            let ty = &item.ty;
            quote! { #ty }
        }),
        None => (
            {
                let params = &generics.params;
                quote! { <_E, #params> }
            },
            quote! { _E },
        ),
    };

    let (error_generics, error_env) = match item.items.iter().find_map(|item| match item {
        ImplItem::Type(ty) if ty.ident == "Error" => Some(ty),
        _ => None,
    }) {
        Some(item) => (quote! { #generics }, {
            let ty = &item.ty;
            quote! { #ty }
        }),
        None => (
            {
                let params = &generics.params;
                quote! { <_E, #params> }
            },
            quote! { _E },
        ),
    };

    let fields = item
        .items
        .into_iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .map(kono_impl_method)
        .collect::<Vec<_>>();

    let inline_schemas = fields
        .iter()
        .map(|field| {
            let ty = match &field.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => quote! { #ty },
            };

            let arguments = field.inputs.iter().map(|input| {
                let ty = &input.pat_type.ty;

                match input.attrs.flatten.is_present() {
                    true => quote! {},
                    false => quote_spanned! { ty.span() =>
                        .chain(<#ty as kono::aspect::InputType<_>>::schema(_environment).into_iter())
                    }
                }
            }).collect::<Vec<_>>();

            quote! {
                #(#arguments)*
                .chain(<#ty as kono::aspect::OutputType<_>>::inline_schema(_environment).into_iter())
            }
        })
        .collect::<Vec<_>>();

    let query_names = fields
        .iter()
        .filter(|field| field.ty == FieldTy::Query)
        .map(|field| field.name.to_owned())
        .collect::<Vec<_>>();

    let query_schema = schema(fields.iter().filter(|field| field.ty == FieldTy::Query));
    let query_handlers = handlers(
        fields.iter().filter(|field| field.ty == FieldTy::Query),
        |ident, args| quote! { Self::#ident(#args) },
    );

    let field_names = fields
        .iter()
        .filter(|field| field.ty == FieldTy::Field)
        .map(|field| field.name.to_owned())
        .collect::<Vec<_>>();

    let field_schema = schema(fields.iter().filter(|field| field.ty == FieldTy::Field));

    let field_handlers = handlers(
        fields.iter().filter(|field| field.ty == FieldTy::Field),
        |ident, args| quote! {self.#ident(#args) },
    );

    let methods = fields
        .into_iter()
        .map(|field| field.method)
        .collect::<Vec<_>>();

    let query_impl = match query_handlers.is_empty() {
        true => quote! {},
        false => quote! {
            fn can_query(_environment: &Self::Environment, field: &str, _context: &Self::Context) -> bool {
                match field {
                    #(#query_names)|* => true,
                    _ => false,
                }
            }

            fn query<'a>(
                field: &'a str,
                args: std::collections::HashMap<String, kono::executor::Value>,
                context: &'a Self::Context,
                _environment: &'a Self::Environment,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                        Output = Result<
                            kono::executor::Intermediate<kono::aspect::ObjectValue>,
                            Self::Error,
                        >,
                    > + 'a,
                >,
            > {
                use kono::aspect::OutputType;

                Box::pin(async move { match field {
                    #(#query_handlers)*
                    _ => unreachable!(),
                } })
            }
        },
    };

    let field_impl = match field_handlers.is_empty() {
        true => quote! {},
        false => quote! {
            fn can_resolve_field(&self, field: &str) -> bool {
                match field {
                    #(#field_names)|* => true,
                    _ => false,
                }
            }

            fn resolve_field<'a>(
                &'a self,
                field: &'a str,
                args: &'a std::collections::HashMap<String, kono::executor::Value>,
                context: &'a Self::Context,
                _environment: &'a Self::Environment,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                        Output = Result<
                            kono::executor::Intermediate<kono::aspect::ObjectValue>,
                            Self::Error,
                        >,
                    > + 'a,
                >,
            > {
                use kono::aspect::OutputType;

                Box::pin(async move { match field {
                    #(#field_handlers)*
                    _ => unreachable!(),
                } })
            }
        },
    };

    Ok(quote! {
        impl #generics #self_ty #where_clause {
            #(#methods)*
        }

        impl #generics kono::executor::Typename for #self_ty #where_clause {
            fn typename(&self) -> std::borrow::Cow<str> {
                #name.into()
            }
        }

        impl #generics kono::aspect::Aspect for #self_ty #where_clause {
            #context
            #environment
            #error

            #field_impl

            #query_impl
        }

        impl #schema_generics kono::aspect::OutputType<#schema_env> for #self_ty #where_clause {
            fn ty(_environment: &#schema_env) -> kono::schema::Type {
                kono::schema::Type::Named(#name.into())
            }

            fn schema(_environment: &#schema_env) -> Vec<kono::schema::Item> {
                vec![
                    kono::schema::ItemType::new(#name)
                        .fields(kono::schema::Fields::Named(vec![#(#field_schema)*]))
                        .into(),
                    kono::schema::ItemType::new("Query")
                        .fields(kono::schema::Fields::Named(vec![#(#query_schema)*]))
                        .into()
                ]
                .into_iter()
                #(#inline_schemas)*
                .collect()
            }

            fn into_intermediate(self, _environment: &#schema_env) -> Result<
                kono::executor::Intermediate<kono::aspect::ObjectValue>,
                kono::aspect::Error,
            > {
                Ok(kono::executor::Intermediate::Object(kono::aspect::ObjectValue::Aspect(Box::new(self))))
            }
        }
    })
}

/// Attribute that can be applied to an `impl` to turn it into a GraphQL type.
///
/// ### Example:
/// ```rust ignore
/// pub struct User;
///
/// #[kono]
/// impl User {
///     fn name(&self) -> &str {
///         "Tim"
///     }
/// }
///
/// server(User::resolver(), || ());
/// ```
#[proc_macro_attribute]
pub fn kono(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let input = parse_macro_input!(item as Item);

    match kono_impl(KonoImpl::from_list(&args).unwrap(), input) {
        Ok(result) => quote! { #result }.into(),
        Err(error) => quote! { compile_error!(#error) }.into(),
    }
}

#[proc_macro_derive(Kono, attributes(kono))]
pub fn kono_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive::kono_derive(item)
}
