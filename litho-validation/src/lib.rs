use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_language::lex::Span;
use litho_types::Database;

mod arguments;
mod directives;
mod enums;
mod extensions;
mod fields;
mod inputs;
mod interfaces;
mod names;
mod types;
mod unions;
mod values;

pub fn check<N, T>(document: &N, database: &Database<T>) -> Vec<Diagnostic<Span>>
where
    N: Node<T>,
    T: Eq + Hash + Borrow<str> + ToString,
{
    let mut errors = vec![];
    document.traverse(&arguments::ArgumentNameUniqueness(database), &mut errors);
    document.traverse(&arguments::ArgumentsAreInputTypes(database), &mut errors);
    document.traverse(
        &directives::SelfReferentialDirectives(database),
        &mut errors,
    );
    document.traverse(&enums::EnumValues(database), &mut errors);
    document.traverse(&extensions::SameTypeExtensions(database), &mut errors);
    document.traverse(&fields::FieldNameUniqueness(database), &mut errors);
    document.traverse(&fields::FieldsAreOutputTypes(database), &mut errors);
    document.traverse(&fields::HasFields(database), &mut errors);
    document.traverse(&inputs::SelfReferentialInputs(database), &mut errors);
    document.traverse(&interfaces::ImplementsInterface(database), &mut errors);
    document.traverse(&names::ReservedNames(database), &mut errors);
    document.traverse(&names::UniqueNames(database), &mut errors);
    document.traverse(&types::NamedTypesExist(database), &mut errors);
    document.traverse(&unions::UnionMemberTypes(database), &mut errors);
    document.traverse(&values::EnumCoercion(database), &mut errors);
    document.traverse(&values::InputCoercion(database), &mut errors);
    document.traverse(&values::ObjectCoercion(database), &mut errors);
    errors
}
