use inflections::Inflect;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg, Ident, ImplItem, ImplItemMethod, Item, Pat, Type};

#[derive(PartialEq, Eq)]
enum FieldTy {
    Mutation,
    Object,
    Query,
}

struct Field {
    ident: Ident,
    name: String,
    has_receiver: bool,
    has_environment: bool,
    method: ImplItemMethod,
    ty: FieldTy,
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

fn kono_impl_method(method: ImplItemMethod) -> Field {
    let ty = match method.attrs[0].to_token_stream().to_string().as_ref() {
        "#[kono :: query]" => FieldTy::Query,
        "#[kono :: field]" => FieldTy::Object,
        _ => todo!(
            "Unknown attr: {:?}",
            method.attrs[0].to_token_stream().to_string()
        ),
    };

    let mut method = method;
    method.attrs = vec![];

    let ident = method.sig.ident.to_owned();
    let name = method
        .sig
        .ident
        .to_string()
        .trim_start_matches("r#")
        .to_owned();

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

    Field {
        ident,
        name,
        has_receiver,
        has_environment,
        method,
        ty,
    }
}

fn kono_impl(item: syn::Item) -> Result<proc_macro2::TokenStream, String> {
    let item = match item {
        Item::Impl(item) => item,
        _ => return Err("`kono` only supports implementations.".to_owned()),
    };

    let generics = item.generics.clone();
    let self_ty = item.self_ty.clone();
    let name = match &*self_ty {
        Type::Path(ty) => ty.path.segments.last().unwrap().ident.to_string(),
        _ => todo!(),
    };
    let where_clause = item.generics.where_clause.clone();

    let context = assoc_type(&item, "Context", || quote! { type Context = (); });
    let environment = assoc_type(&item, "Environment", || quote! { type Environment = (); });
    let error = assoc_type(
        &item,
        "Error",
        || quote! { type Error = kono_aspect::Error; },
    );

    let fields = item
        .items
        .into_iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .map(kono_impl_method)
        .collect::<Vec<_>>();

    let query_names = fields
        .iter()
        .filter(|field| field.ty == FieldTy::Query)
        .map(|field| field.name.to_camel_case())
        .collect::<Vec<_>>();

    let query_handlers = fields
        .iter()
        .filter(|field| field.ty == FieldTy::Query)
        .map(|field| {
            let name = field.name.to_camel_case();
            let ident = field.ident.to_owned();

            let mut args = vec![];

            if field.has_environment {
                args.push(quote! { environment });
            }

            quote! {
                #name => Self::#ident(#(#args),*).into_intermediate(),
            }
        })
        .collect::<Vec<_>>();

    let field_names = fields
        .iter()
        .filter(|field| field.ty == FieldTy::Object)
        .map(|field| field.name.to_camel_case())
        .collect::<Vec<_>>();

    let field_handlers = fields
        .iter()
        .filter(|field| field.ty == FieldTy::Object)
        .map(|field| {
            let name = field.name.to_camel_case();
            let ident = field.ident.to_owned();

            quote! {
                #name => self.#ident().into_intermediate(),
            }
        })
        .collect::<Vec<_>>();

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
                environment: &'a Self::Environment,
                field: &'a str,
                args: std::collections::HashMap<String, kono_executor::Value>,
                context: &'a Self::Context
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                        Output = Result<
                            kono_executor::Intermediate<kono_aspect::ObjectValue>,
                            Self::Error,
                        >,
                    > + 'a,
                >,
            > {
                use kono_aspect::IntoIntermediate;

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
                    "__typename" => true,
                    #(#field_names)|* => true,
                    _ => false,
                }
            }

            fn resolve_field<'a>(
                &'a self,
                field: &'a str,
                args: &'a std::collections::HashMap<String, kono_executor::Value>,
                context: &'a Self::Context
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                        Output = Result<
                            kono_executor::Intermediate<kono_aspect::ObjectValue>,
                            Self::Error,
                        >,
                    > + 'a,
                >,
            > {
                use kono_aspect::IntoIntermediate;

                Box::pin(async move { match field {
                    "__typename" => Ok(kono_executor::Intermediate::Value(#name.into())),
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

        impl #generics kono_aspect::Aspect for #self_ty #where_clause {
            #environment
        }

        impl #generics kono_aspect::ResolveField for #self_ty #where_clause {
            #context
            #error

            fn typename(&self) -> &str {
                #name
            }

            #field_impl
        }

        impl #generics kono_aspect::Query for #self_ty #where_clause {
            #context
            #environment
            #error

            #query_impl
        }

        impl #generics kono_aspect::Mutation for #self_ty #where_clause {
            #context
            #error
        }

        impl #generics kono_aspect::IntoIntermediate<
            <Self as kono_aspect::ResolveField>::Error,
        > for #self_ty #where_clause {
            fn into_intermediate(self) -> Result<
                kono_executor::Intermediate<kono_aspect::ObjectValue>,
                <Self as kono_aspect::ResolveField>::Error,
            > {
                Ok(kono_executor::Intermediate::Object(kono_aspect::ObjectValue::Aspect(Box::new(self))))
            }
        }
    })
}

#[proc_macro_attribute]
pub fn kono(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as Item);

    match kono_impl(input) {
        Ok(result) => quote! { #result }.into(),
        Err(error) => quote! { compile_error!(#error) }.into(),
    }
}
