mod common;
mod query;
mod schema;

pub use common::*;
use graphql_parser::Pos;
pub use query::*;
pub use schema::*;

#[derive(Clone, Copy, Debug)]
pub struct Span(pub Pos, pub usize);

impl Default for Span {
    fn default() -> Self {
        Span(Pos { line: 1, column: 1 }, 0)
    }
}
