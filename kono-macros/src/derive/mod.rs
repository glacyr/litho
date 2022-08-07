use quote::quote;
use syn::{parse_macro_input, Item};

mod enums;
mod structs;

pub fn kono_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as Item);

    let result = match input {
        Item::Enum(item) => enums::kono_derive_enum(item),
        Item::Struct(item) => structs::kono_derive_struct(item),
        _ => todo!("only enums and structs are supported right now"),
    };

    match result {
        Ok(result) => quote! { #result }.into(),
        Err(error) => quote! { compile_error!(#error) }.into(),
    }
}
