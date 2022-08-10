use darling::util::Flag;
use darling::FromAttributes;
use inflections::Inflect;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Fields, FieldsNamed, FieldsUnnamed, ItemStruct};

#[derive(Debug, FromAttributes)]
#[darling(attributes(kono))]
struct Kono {
    input: Flag,
    output: Flag,
}

pub fn kono_derive_struct_input(item: &ItemStruct) -> Result<proc_macro2::TokenStream, String> {
    let self_ty = &item.ident;
    let name = self_ty.to_string() + "Input";

    let mut field_schemas = vec![];
    let mut inline_schemas = vec![];

    let (names, fields) = match &item.fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) => fields
            .into_iter()
            .enumerate()
            .map(|(index, field)| {
                let ident = &field.ident;

                let name = field
                    .ident
                    .as_ref()
                    .map(|ident| ident.to_string())
                    .unwrap_or(format!("_{}", index))
                    .to_camel_case();

                let ty = &field.ty;

                field_schemas.push(quote_spanned! { ty.span() =>
					.field(kono::schema::InputValue::new(#name, <#ty as kono::aspect::InputType<Env>>::ty(_environment)))
				});

                inline_schemas.push(quote_spanned! { ty.span() =>
                    .chain(<#ty as kono::aspect::InputType<Env>>::schema(_environment).into_iter())
                });

                return (
                    quote! { #ident },
                    quote_spanned! { ty.span() =>
                        let #ident = kono::aspect::InputType::<Env>::from_value_option(object.get(#name).cloned())?;
                    },
                );
            })
            .unzip(),
        _ => (vec![], vec![]),
    };

    Ok(quote! {
        impl<Env> kono::aspect::InputType<Env> for #self_ty {
            fn ty(_environment: &Env) -> kono::schema::Type {
                kono::schema::Type::Named(#name.into())
            }

            fn schema(_environment: &Env) -> Vec<kono::schema::Item> {
                std::iter::once(
                    kono::schema::ItemInput::new(#name)
                    #(#field_schemas)*
                    .into()
                )
                    #(#inline_schemas)*
                    .collect()
            }

            fn from_value<E>(value: kono::executor::Value) -> Result<Self, E>
            where
                Self: Sized,
                E: kono::executor::Error {
                match value {
                    kono::executor::Value::Object(object) => {
                        #(#fields)*

                        Ok(Self { #(#names),* })
                    },
                    _ => Err(E::unexpected_value_type(#name)),
                }
            }
        }
    })
}

pub fn kono_derive_struct_output(item: &ItemStruct) -> Result<proc_macro2::TokenStream, String> {
    let self_ty = &item.ident;
    let name = self_ty.to_string();

    let mut field_schemas = vec![];
    let mut inline_schemas = vec![];

    let fields = match &item.fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) => fields
            .into_iter()
            .enumerate()
            .map(|(index, field)| {
                let ident = &field.ident;

                let name = field
                    .ident
                    .as_ref()
                    .map(|ident| ident.to_string())
                    .unwrap_or(format!("_{}", index))
                    .to_camel_case();

                let ty = &field.ty;

                field_schemas.push(quote! {
					kono::schema::Field::new(Some(#name), <#ty as kono::aspect::OutputType<_>>::ty(_environment)),
				});

                inline_schemas.push(quote! {
                    .chain(<#ty as kono::aspect::OutputType<_>>::inline_schema(_environment).into_iter())
                });

                return quote! {
                    map.insert(#name.to_owned(), self.#ident.into_intermediate(_environment)?);
                };
            })
            .collect(),
        _ => vec![],
    };

    Ok(quote! {
        impl<Env> kono::aspect::OutputType<Env> for #self_ty {
            fn ty(_environment: &Env) -> kono::schema::Type {
                kono::schema::Type::Named(#name.into())
            }

            fn inline(_environment: &Env) -> bool {
                true
            }

            fn schema(_environment: &Env) -> Vec<kono::schema::Item> {
                std::iter::once(
                    kono::schema::ItemType::new(#name)
                    .fields(kono::schema::Fields::Named(vec![
                        #(#field_schemas)*
                    ]))
                    .into()
                )
                    #(#inline_schemas)*
                    .collect()
            }

            fn into_intermediate(self, _environment: &Env) -> Result<
                kono::executor::Intermediate<kono::aspect::ObjectValue>,
                kono::aspect::Error,
            > {
                let mut map = std::collections::HashMap::new();
                #(#fields)*
                Ok(kono::executor::Intermediate::Object(
                    kono::aspect::ObjectValue::Record(
                        kono::aspect::Record::new(#name, map),
                    ),
                ))
            }
        }
    })
}

pub fn kono_derive_struct(item: ItemStruct) -> Result<proc_macro2::TokenStream, String> {
    let kono = Kono::from_attributes(&item.attrs).unwrap();

    let (is_input, is_output) = match (kono.input.is_present(), kono.output.is_present()) {
        (false, false) => (false, true),
        flags => flags,
    };

    let input = if is_input {
        kono_derive_struct_input(&item)?
    } else {
        quote! {}
    };

    let output = if is_output {
        kono_derive_struct_output(&item)?
    } else {
        quote! {}
    };

    Ok(vec![input, output].into_iter().collect())
}
