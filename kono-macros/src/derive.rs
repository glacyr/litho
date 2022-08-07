use inflections::Inflect;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

fn kono_derive_impl(item: ItemEnum) -> Result<proc_macro2::TokenStream, String> {
    let self_ty = item.ident;
    let name = self_ty.to_string();

    let mut variants = vec![];

    for variant in item.variants.iter() {
        let name = &variant.ident;
        let value = name.to_string().to_constant_case();

        variants.push(quote! { Self::#name => #value, });
    }

    Ok(quote! {
        impl<E> kono::IntoIntermediate<E> for #self_ty {
            fn into_intermediate(self) -> Result<kono::Intermediate<kono::ObjectValue>, E> {
                match self {
                    #(#variants)*
                }.into_intermediate()
            }
        }

        impl<E> kono::OutputType<E> for #self_ty {
            fn ty(_environment: &E) -> kono::Type {
                kono::Type::Named(#name.into())
            }

            fn schema(_environment: &E) -> Vec<kono::Item> {
                vec![kono::ItemEnum::new(#name).into()]
            }
        }
    })
}

pub fn kono_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    match kono_derive_impl(input) {
        Ok(result) => quote! { #result }.into(),
        Err(error) => quote! { compile_error!(#error) }.into(),
    }
}
