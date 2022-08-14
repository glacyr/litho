use darling::FromMeta;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{AttributeArgs, ImplItem, ItemImpl, LitStr, Type};

use super::{Field, FieldTy};

#[derive(Debug, FromMeta)]
struct KonoAttrs {
    rename: Option<LitStr>,
}

pub struct Aspect {
    attrs: KonoAttrs,
    item: ItemImpl,
    fields: Vec<Field>,
}

impl Aspect {
    pub fn new(args: AttributeArgs, item: ItemImpl) -> Aspect {
        let fields = item
            .items
            .clone()
            .into_iter()
            .filter_map(|item| match item {
                ImplItem::Method(method) => Some(method),
                _ => None,
            })
            .map(Field::new)
            .collect::<Vec<_>>();

        Aspect {
            attrs: KonoAttrs::from_list(&args).unwrap(),
            item,
            fields,
        }
    }

    fn assoc_type(item: &ItemImpl, name: &str, default: fn() -> TokenStream) -> TokenStream {
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

    pub fn fields(&self) -> impl Iterator<Item = &Field> + Clone {
        self.fields
            .iter()
            .filter(|field| field.ty() == FieldTy::Field)
    }

    pub fn mutations(&self) -> impl Iterator<Item = &Field> + Clone {
        self.fields
            .iter()
            .filter(|field| field.ty() == FieldTy::Mutation)
    }

    pub fn queries(&self) -> impl Iterator<Item = &Field> + Clone {
        self.fields
            .iter()
            .filter(|field| field.ty() == FieldTy::Query)
    }

    pub fn emit(self) -> TokenStream {
        vec![self.schema()].into_iter().collect()
    }

    pub fn inline_schema(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.fields.iter().map(Field::inline_schema)
    }

    pub fn handler<'a>(
        &self,
        name: &str,
        fields: impl Iterator<Item = &'a Field> + Clone,
        has_receiver: bool,
    ) -> TokenStream {
        vec![
            self.handler_can(name, fields.clone()),
            self.handler_do(name, fields, has_receiver),
        ]
        .into_iter()
        .collect()
    }

    pub fn handler_can<'a>(
        &self,
        name: &str,
        fields: impl Iterator<Item = &'a Field>,
    ) -> TokenStream {
        let ident = Ident::new(&format!("can_{}", name), Span::call_site());

        let names = fields
            .map(|field| field.name().to_owned())
            .collect::<Vec<_>>();

        if names.is_empty() {
            return Default::default();
        }

        quote! {
            fn #ident(
                field: &str,
                _context: &Self::Context,
                _environment: &Self::Environment,
            ) -> bool {
                match field {
                    #(#names)|* => true,
                    _ => false,
                }
            }
        }
    }

    pub fn handler_do<'a>(
        &self,
        name: &str,
        fields: impl Iterator<Item = &'a Field>,
        has_receiver: bool,
    ) -> TokenStream {
        let ident = Ident::new(name, Span::call_site());

        let handlers = fields.map(Field::handler).collect::<Vec<_>>();

        let receiver = match has_receiver {
            true => quote! { &'a self, },
            false => quote! {},
        };

        quote! {
            fn #ident<'a>(
                #receiver
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
                    #(#handlers)*
                    _ => unreachable!(),
                } })
            }
        }
    }

    pub fn schema(self) -> TokenStream {
        let generics = self.item.generics.clone();
        let self_ty = self.item.self_ty.clone();
        let name = self
            .attrs
            .rename
            .as_ref()
            .cloned()
            .unwrap_or(match &*self_ty {
                Type::Path(ty) => {
                    let ident = &ty.path.segments.last().unwrap().ident;
                    LitStr::new(&ident.to_string(), ident.span())
                }
                _ => todo!(),
            });
        let where_clause = self.item.generics.where_clause.clone();

        let context = Self::assoc_type(&self.item, "Context", || quote! { type Context = (); });
        let environment = Self::assoc_type(
            &self.item,
            "Environment",
            || quote! { type Environment = (); },
        );
        let error = Self::assoc_type(
            &self.item,
            "Error",
            || quote! { type Error = kono::aspect::Error; },
        );

        let (schema_generics, schema_env) = match self.item.items.iter().find_map(|item| match item
        {
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

        let inline_schema = self.inline_schema().collect::<Vec<_>>();

        let resolve_field = self.handler("resolve_field", self.fields(), true);
        let mutate = self.handler("mutate", self.mutations(), false);
        let query = self.handler("query", self.queries(), false);

        let field_schema = self.fields().map(Field::schema).collect::<Vec<_>>();
        let mutation_schema = self.mutations().map(Field::schema).collect::<Vec<_>>();
        let query_schema = self.queries().map(Field::schema).collect::<Vec<_>>();

        let mutation_schema = match mutation_schema.is_empty() {
            true => quote! {},
            false => quote! {
                kono::schema::ItemType::new("Mutation")
                    .fields(kono::schema::Fields::Named(vec![#(#mutation_schema)*]))
                    .into(),
            },
        };

        let methods = self
            .fields
            .into_iter()
            .map(Field::into_method)
            .collect::<Vec<_>>();

        quote! {
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

                #resolve_field
                #mutate
                #query
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
                        #mutation_schema
                    ]
                    .into_iter()
                    #(#inline_schema)*
                    .collect()
                }

                fn into_intermediate(self, _environment: &#schema_env) -> Result<
                    kono::executor::Intermediate<kono::aspect::ObjectValue>,
                    kono::aspect::Error,
                > {
                    Ok(kono::executor::Intermediate::Object(kono::aspect::ObjectValue::Aspect(Box::new(self))))
                }
            }
        }
    }
}
