use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_language::lex::Span;
use litho_types::Database;

mod arguments;
mod directives;
mod values;
mod variables;

pub fn check<N, T>(document: &N, database: &Database<T>) -> Vec<Diagnostic<Span>>
where
    N: Node<T>,
    T: Eq + Hash + Borrow<str> + ToString,
{
    let mut errors = vec![];
    document.traverse(&arguments::ArgumentNames(database), &mut errors);
    document.traverse(&arguments::ArgumentUniqueness(database), &mut errors);
    document.traverse(&arguments::RequiredArguments(database), &mut errors);
    document.traverse(&directives::DirectivesAreDefined(database), &mut errors);
    document.traverse(
        &directives::DirectivesAreInValidLocations(database),
        &mut errors,
    );
    document.traverse(
        &directives::DirectivesAreUniquePerLocation(database),
        &mut errors,
    );
    document.traverse(&values::EnumCoercion(database), &mut errors);
    document.traverse(&values::InputCoercion(database), &mut errors);
    document.traverse(&values::ObjectCoercion(database), &mut errors);
    document.traverse(&variables::VariableUniqueness(database), &mut errors);
    document.traverse(&variables::VariablesAreInputTypes(database), &mut errors);
    errors
}
