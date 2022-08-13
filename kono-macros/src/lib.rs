use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, AttributeArgs, ImplItem, Item, LitStr, Type};

mod aspect;
mod derive;

use aspect::{Field, FieldTy};

#[derive(Debug, FromMeta)]
struct KonoImpl {
    rename: Option<LitStr>,
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
        .map(Field::new)
        .collect::<Vec<_>>();

    let inline_schemas = fields.iter().map(Field::inline_schema).collect::<Vec<_>>();

    let query_names = fields
        .iter()
        .filter(|field| field.ty() == FieldTy::Query)
        .map(|field| field.name().to_owned())
        .collect::<Vec<_>>();

    let query_schema = fields
        .iter()
        .filter(|field| field.ty() == FieldTy::Query)
        .map(Field::schema)
        .collect::<Vec<_>>();

    let query_handlers = fields
        .iter()
        .filter(|field| field.ty() == FieldTy::Query)
        .map(Field::handler)
        .collect::<Vec<_>>();

    let field_names = fields
        .iter()
        .filter(|field| field.ty() == FieldTy::Field)
        .map(|field| field.name().to_owned())
        .collect::<Vec<_>>();

    let field_schema = fields
        .iter()
        .filter(|field| field.ty() == FieldTy::Field)
        .map(Field::schema)
        .collect::<Vec<_>>();

    let field_handlers = fields
        .iter()
        .filter(|field| field.ty() == FieldTy::Field)
        .map(Field::handler)
        .collect::<Vec<_>>();

    let methods = fields
        .into_iter()
        .map(Field::into_method)
        .collect::<Vec<_>>();

    let query_impl = match query_handlers.is_empty() {
        true => quote! {},
        false => quote! {
            fn can_query(
                field: &str,
                _context: &Self::Context,
                _environment: &Self::Environment,
            ) -> bool {
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
            fn can_resolve_field(
                field: &str,
                _context: &Self::Context,
                _environment: &Self::Environment,
            ) -> bool {
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
                        .into(),
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
