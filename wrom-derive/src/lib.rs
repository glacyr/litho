use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, PathArguments, ReturnType, Type, TypeParamBound};

#[proc_macro_attribute]
pub fn wrom(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = item;

    let ReturnType::Type(_, ty) = &sig.output else {
        todo!();
    };

    let Type::ImplTrait(ty) = ty.as_ref() else {
        todo!();
    };

    let TypeParamBound::Trait(ty) = ty.bounds.first().unwrap() else {
        todo!()
    };

    let PathArguments::AngleBracketed(ty) = &ty.path.segments.last().unwrap().arguments else {
        todo!();
    };

    let generics = &sig.generics.params;
    let params = &sig.inputs;
    let param_names = params
        .iter()
        .map(|param| match param {
            FnArg::Typed(param) => &param.pat,
            _ => todo!(),
        })
        .collect::<Vec<_>>();
    let input = &ty.args[0];
    let output = &ty.args[1];
    let error = &ty.args[2];

    let where_clause = sig.generics.where_clause.as_ref();

    let lifetimes = sig.generics.lifetimes();
    let named_generics = sig.generics.type_params().collect::<Vec<_>>();

    quote! {
        #(#attrs)*
        #vis #sig {
            // #block
            pub struct Parser < #generics >
            #where_clause {
                marker: ::std::marker::PhantomData<(
                    #(&#lifetimes)*
                    #(#named_generics,)*
                )>,
                #params
            };

            impl < #generics > ::wrom::RecoverableParser<#input, #output, #error> for Parser < #generics >
            #where_clause {
                #[inline(always)]
                fn recognizer(&self) -> #input::Recognizer {
                    let Parser {
                        #(#param_names,)*
                        ..
                    } = self;

                    #(
                        let #param_names = #param_names.clone();
                    )*

                    let parser = #block;

                    <_ as ::wrom::RecoverableParser<#input, #output, #error>>::recognizer(&parser)
                }

                #[inline(always)]
                fn parse(&mut self, input: &mut #input, recovery_point: #input::Recognizer) -> ::std::result::Result<#output, #error>
                {
                    let Parser {
                        #(#param_names,)*
                        ..
                    } = self;

                    #(
                        let #param_names = #param_names.clone();
                    )*

                    #block.parse(input, recovery_point)
                }
            }

            Parser {
                #(#param_names,)*
                marker: ::std::marker::PhantomData,
            }
        }
    }
    .into()
}
