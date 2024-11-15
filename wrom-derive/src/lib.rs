use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Expr, FnArg, ItemFn, PathArguments, ReturnType, Type, TypeParamBound,
};

#[proc_macro_attribute]
pub fn wrom(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    let recognizer = parse_macro_input!(attrs as Expr);

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

    quote! {
        #(#attrs)*
        #vis #sig {
            pub struct Parser {
                #params
            };

            impl < #generics > ::wrom::Recognizer<#input, #error> for Parser
            #where_clause
            {
                #[inline(always)]
                fn recognize(&self, input: #input) -> ::nom::IResult<#input, (), Error> {
                    let Parser {
                        #(#param_names,)*
                    } = *self;

                    #recognizer.recognize(input)
                }
            }

            impl < #generics > ::wrom::RecoverableParser<#input, #output, #error> for Parser
            #where_clause {
                #[inline(always)]
                fn parse<R>(&self, input: #input, recovery_point: R) -> ::nom::IResult<#input, #output, #error>
                where
                    R: ::wrom::Recognizer<#input, #error>,
                {
                    let Parser {
                        #(#param_names,)*
                    } = *self;

                    #block.parse(input, recovery_point)
                }
            }

            Parser {
                #(#param_names,)*
            }
        }
    }
    .into()
}
