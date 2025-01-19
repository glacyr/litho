// #![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod ast;
pub mod chk;
pub mod fmt;
pub mod lex;
pub mod syn;

pub use ast::Document;
pub use syn::Parse;
