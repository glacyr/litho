// #![warn(missing_docs)]

pub mod ast;
pub mod chk;
pub mod lex;
pub mod syn;

pub use ast::Document;
pub use syn::{parse_from_str, Parse};
