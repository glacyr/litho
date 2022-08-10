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
        impl<Env> kono::aspect::OutputType<Env> for #self_ty {
            fn ty(_environment: &Env) -> kono::schema::Type {
                kono::schema::Type::Named(#name.into())
            }

            fn inline(_environment: &Env) -> bool {
                true
            }

            fn schema(_environment: &Env) -> Vec<kono::schema::Item> {
                vec![kono::schema::ItemEnum::new(#name)
                    #(#variants)*
                    .into()]
            }

            fn into_intermediate(self, environment: &Env) -> Result<
                kono::executor::Intermediate<kono::aspect::ObjectValue>,
                kono::aspect::Error
            > {
                match self {
                    #(#serializers)*
                }.into_intermediate(environment)
            }
        }
    })
}
