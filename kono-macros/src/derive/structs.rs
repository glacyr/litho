use inflections::Inflect;
use quote::quote;
use syn::{Fields, FieldsNamed, FieldsUnnamed, ItemStruct};

pub fn kono_derive_struct(item: ItemStruct) -> Result<proc_macro2::TokenStream, String> {
    let self_ty = item.ident;
    let name = self_ty.to_string();

    let generics = &item.generics.params;
    let where_clause = &item.generics.where_clause;

    let mut field_schemas = vec![];
    let mut inline_schemas = vec![];

    let fields = match item.fields {
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
                    map.insert(#name.to_owned(), self.#ident.into_intermediate()?);
                };
            })
            .collect(),
        _ => vec![],
    };

    Ok(quote! {
        impl #generics kono::aspect::IntoIntermediate<kono::aspect::Error> for #self_ty #where_clause {
            fn into_intermediate(self) -> Result<
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

        impl<E> kono::aspect::OutputType<E> for #self_ty {
            fn ty(_environment: &E) -> kono::schema::Type {
                kono::schema::Type::Named(#name.into())
            }

            fn inline(_environment: &E) -> bool {
                true
            }

            fn schema(_environment: &E) -> Vec<kono::schema::Item> {
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
        }
    })
}
