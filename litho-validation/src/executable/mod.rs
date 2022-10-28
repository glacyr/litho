use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_language::lex::Span;
use litho_types::Database;

mod fields;
mod fragments;
mod operations;

pub fn check<N, T>(document: &N, database: &Database<T>) -> Vec<Diagnostic<Span>>
where
    N: Node<T>,
    T: Eq + Hash + Borrow<str> + ToString,
{
    let mut errors = vec![];
    document.traverse(&fields::FieldSelectionMerging(database), &mut errors);
    document.traverse(&fields::FieldSelections(database), &mut errors);
    document.traverse(&fields::LeafFieldSelections(database), &mut errors);
    document.traverse(&fragments::FragmentNameUniqueness(database), &mut errors);
    document.traverse(&fragments::FragmentOnCompositeTypes(database), &mut errors);
    document.traverse(&fragments::FragmentsMustBeUsed(database), &mut errors);
    document.traverse(&operations::OperationNameUniqueness(database), &mut errors);
    document.traverse(&operations::LoneAnonymousOperation(database), &mut errors);
    errors
}
