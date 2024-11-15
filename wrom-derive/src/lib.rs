use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Expr, FnArg, ItemFn, PathArguments, ReturnType, Type, TypeParamBound,
};

#[proc_macro_attribute]
pub fn wrom(attrs: TokenStream, input: TokenStream) -> TokenStream {
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
    let recovery_point = &ty.args[3];

    let where_clause = sig.generics.where_clause.as_ref();

    let name = &sig.ident;

    quote! {
        #(#attrs)*
        #vis #sig {
            pub struct Parser {
                #params
            };

            impl < #generics > ::wrom::RecoverableParser<#input, #output, #error, #recovery_point> for Parser
            #where_clause {
                fn recovery_point(&self) -> #recovery_point {
                    let Parser {
                        #(#param_names,)*
                    } = *self;

                    let parser = #block;

                    <_ as ::wrom::RecoverableParser<#input, #output, #error, #recovery_point>>::recovery_point(&parser)
                }

                fn parse(&self, input: #input, recovery_point: #recovery_point) -> ::nom::IResult<#input, #output, #error>
                {
                    let Parser {
                        #(#param_names,)*
                    } = *self;

                    // eprintln!("{}: recovery = {:?}", stringify!(#name), recovery_point);

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
