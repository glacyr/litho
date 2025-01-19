use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_language::lex::Span;
use litho_types::Database;

mod common;
mod executable;
mod system;

pub fn check<'a, N, T>(document: &N, database: &Database<'a, T>) -> Vec<Diagnostic<Span>>
where
    N: Node<'a, T>,
    T: ContextValue<'a> + Eq + Hash + Borrow<str> + ToString,
{
    let mut errors = system::check(document, database);
    errors.extend(executable::check(document, database));
    errors.extend(common::check(document, database));
    errors
}
