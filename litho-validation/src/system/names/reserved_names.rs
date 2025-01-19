use std::borrow::Borrow;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ReservedNames<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for ReservedNames<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + Borrow<str> + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_input_value_definition(
        &self,
        node: &'ast Shared<'a, T, InputValueDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if <T as Borrow<str>>::borrow(node.name.as_ref()).starts_with("__") {
            accumulator.push(Diagnostic::reserved_input_value_name(
                node.name.as_ref().to_string(),
                node.name.span(),
            ))
        }
    }

    fn visit_field_definition(
        &self,
        node: &'ast Shared<'a, T, FieldDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if <T as Borrow<str>>::borrow(node.name.as_ref()).starts_with("__") {
            accumulator.push(Diagnostic::reserved_field_name(
                node.name.as_ref().to_string(),
                node.name.span(),
            ));
        }
    }

    fn visit_directive_definition(
        &self,
        node: &'ast Shared<'a, T, DirectiveDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.name.ok() {
            Some(name) if <T as Borrow<str>>::borrow(name.as_ref()).starts_with("__") => {
                accumulator.push(Diagnostic::reserved_directive_name(
                    name.as_ref().to_string(),
                    node.name.span(),
                ))
            }
            _ => {}
        }
    }
}
