use darling::util::Flag;
use darling::FromAttributes;
use inflections::Inflect;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Pat, PatType};

#[derive(Debug, FromAttributes)]
#[darling(attributes(kono))]
pub struct InputAttrs {
    flatten: Flag,
}

pub struct Input {
    attrs: InputAttrs,
    pat_type: PatType,
}

impl Input {
    pub fn new(pat_type: PatType) -> Input {
        Input {
            attrs: InputAttrs::from_attributes(&pat_type.attrs).unwrap(),
            pat_type,
        }
    }

    pub fn inline_schema(&self) -> TokenStream {
        let ty = &self.pat_type.ty;

        match self.attrs.flatten.is_present() {
            true => quote! {},
            false => quote_spanned! { ty.span() =>
                .chain(<#ty as kono::aspect::InputType<_>>::schema(_environment).into_iter())
            },
        }
    }

    pub fn schema(&self) -> TokenStream {
        let name = match &*self.pat_type.pat {
            Pat::Ident(pat) => pat.ident.to_string(),
            _ => unreachable!(),
        };

        let ty = &self.pat_type.ty;

        match self.attrs.flatten.is_present() {
            true => quote_spanned! { ty.span() =>
                .arguments(<#ty as kono::aspect::ArgumentType<_>>::schema(_environment))
            },
            false => quote_spanned! { ty.span() =>
                .argument(kono::schema::InputValue::new(#name, <#ty as kono::aspect::InputType<_>>::ty(_environment)))
            },
        }
    }

    pub fn handler(&self) -> TokenStream {
        let name = match &*self.pat_type.pat {
            Pat::Ident(pat) => pat.ident.to_string().to_camel_case(),
            _ => unreachable!(),
        };

        let ty = &self.pat_type.ty;
        match self.attrs.flatten.is_present() {
            true => quote_spanned! { ty.span() =>
                <#ty as kono::aspect::ArgumentType::<_>>::from_args(&args, _environment)?
            },
            false => quote_spanned! { ty.span() =>
                <#ty as kono::aspect::InputType::<_>>::from_value_option(args.get(#name).cloned(), _environment)?
            },
        }
    }
}
