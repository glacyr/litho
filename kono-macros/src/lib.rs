use syn::{parse_macro_input, AttributeArgs, ItemImpl};

mod aspect;
mod derive;

use aspect::Aspect;

/// Attribute that can be applied to an `impl` to turn it into a GraphQL type.
///
/// ### Example:
/// ```rust ignore
/// pub struct User;
///
/// #[kono]
/// impl User {
///     fn name(&self) -> &str {
///         "Tim"
///     }
/// }
///
/// server(User::resolver(), || ());
/// ```
#[proc_macro_attribute]
pub fn kono(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let input = parse_macro_input!(item as ItemImpl);

    Aspect::new(args, input).emit().into()
}

#[proc_macro_derive(Kono, attributes(kono))]
pub fn kono_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive::kono_derive(item)
}
