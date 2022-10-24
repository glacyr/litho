use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ReservedNames<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ReservedNames<'a, T>
where
    T: Eq + Hash + Borrow<str> + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_input_value_definition(
        &self,
        node: &'a Arc<InputValueDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.name.as_ref().borrow().starts_with("__") {
            accumulator.push(Diagnostic::reserved_input_value_name(
                node.name.as_ref().to_string(),
                node.name.span(),
            ))
        }
    }

    fn visit_field_definition(
        &self,
        node: &'a Arc<FieldDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.name.as_ref().borrow().starts_with("__") {
            accumulator.push(Diagnostic::reserved_field_name(
                node.name.as_ref().to_string(),
                node.name.span(),
            ));
        }
    }

    fn visit_directive_definition(
        &self,
        node: &'a Arc<DirectiveDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.name.ok() {
            Some(name) if name.as_ref().borrow().starts_with("__") => accumulator.push(
                Diagnostic::reserved_directive_name(name.as_ref().to_string(), node.name.span()),
            ),
            _ => {}
        }
    }
}
