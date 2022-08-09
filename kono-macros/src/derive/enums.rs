use inflections::Inflect;
use quote::quote;
use syn::ItemEnum;

pub fn kono_derive_enum(item: ItemEnum) -> Result<proc_macro2::TokenStream, String> {
    let self_ty = item.ident;
    let name = self_ty.to_string();

    let mut serializers = vec![];
    let mut variants = vec![];

    for variant in item.variants.iter() {
        let name = &variant.ident;
        let value = name.to_string().to_constant_case();

        variants.push(quote! { .variant(kono::schema::Variant::new(#value) )});
        serializers.push(quote! { Self::#name => #value, });
    }

    Ok(quote! {
        impl<E> kono::aspect::IntoIntermediate<E> for #self_ty {
            fn into_intermediate(self) -> Result<kono::executor::Intermediate<kono::aspect::ObjectValue>, E> {
                match self {
                    #(#serializers)*
                }.into_intermediate()
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
                vec![kono::schema::ItemEnum::new(#name)
                    #(#variants)*
                    .into()]
            }
        }
    })
}
