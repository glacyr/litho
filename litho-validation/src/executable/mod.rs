use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_language::lex::Span;
use litho_types::Database;

mod operations;

pub fn check<N, T>(document: &N, database: &Database<T>) -> Vec<Diagnostic<Span>>
where
    N: Node<T>,
    T: Eq + Hash + Borrow<str> + ToString,
{
    let mut errors = vec![];
    document.traverse(&operations::OperationNameUniqueness(database), &mut errors);
    errors
}
