use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct NamedTypesExist<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for NamedTypesExist<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_named_type(&self, node: &'a NamedType<T>, accumulator: &mut Self::Accumulator) {
        if self
            .0
            .type_definitions_by_name(node.0.as_ref())
            .next()
            .is_none()
        {
            accumulator.push(Diagnostic::unknown_named_type(
                node.0.as_ref().to_string(),
                node.span(),
            ))
        }
    }
}
