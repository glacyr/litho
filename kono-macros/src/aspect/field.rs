use darling::util::Flag;
use darling::FromAttributes;
use inflections::Inflect;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Attribute, FnArg, Ident, ImplItemMethod, Lit, LitStr, Meta, Pat, ReturnType};

use super::Input;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FieldTy {
    Mutation,
    Field,
    Query,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(kono))]
struct FieldAttrs {
    mutation: Flag,
    rename: Option<LitStr>,
}

pub struct Field {
    ident: Ident,
    name: LitStr,
    has_receiver: bool,
    has_environment: bool,
    doc: Option<Lit>,
    method: ImplItemMethod,
    ty: FieldTy,
    inputs: Vec<Input>,
    output: ReturnType,
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

impl Field {
    pub fn new(method: ImplItemMethod) -> Field {
        let kono = FieldAttrs::from_attributes(&method.attrs).unwrap();
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
            .map(|pat_type| Input::new(pat_type))
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
            has_receiver,
            has_environment,
            doc,
            output,
            inputs,
            method,
            ty,
        }
    }

    pub fn ty(&self) -> FieldTy {
        self.ty
    }

    pub fn name(&self) -> &LitStr {
        &self.name
    }

    pub fn inline_schema(&self) -> TokenStream {
        let ty = match &self.output {
            ReturnType::Default => quote! { () },
            ReturnType::Type(_, ty) => quote! { #ty },
        };

        let arguments = self
            .inputs
            .iter()
            .map(Input::inline_schema)
            .collect::<Vec<_>>();

        quote! {
            #(#arguments)*
            .chain(<#ty as kono::aspect::OutputType<_>>::inline_schema(_environment).into_iter())
        }
    }

    pub fn schema(&self) -> TokenStream {
        let name = &self.name;
        let description = match &self.doc {
            Some(description) => quote! { .description(#description.trim()) },
            _ => quote! {},
        };
        let ty = match &self.output {
            ReturnType::Default => quote! { () },
            ReturnType::Type(_, ty) => quote! { #ty },
        };

        let inputs = self.inputs.iter().map(Input::schema);

        quote_spanned! { ty.span() =>
            kono::schema::Field::new(Some(#name), <#ty as kono::aspect::OutputType<_>>::ty(_environment))
                #description
                #(#inputs)*,
        }
    }

    pub fn handler(&self) -> TokenStream {
        let name = &self.name;
        let ident = self.ident.to_owned();

        let mut args = vec![];

        if self.has_environment {
            args.push(quote! { _environment });
        }

        args.extend(self.inputs.iter().map(Input::handler));

        let im = match self.has_receiver {
            true => quote! { self.#ident(#(#args),*) },
            false => quote! { Self::#ident(#(#args),*) },
        };

        quote! {
            #name => #im.into_intermediate(_environment),
        }
    }

    pub fn into_method(self) -> ImplItemMethod {
        self.method
    }
}
