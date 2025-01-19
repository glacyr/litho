use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct UniqueNames<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for UniqueNames<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_directive_definition(
        &self,
        node: &'ast Shared<'a, T, DirectiveDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        let Some(first) = self.0.directive_definitions_by_name(name.as_ref()).next() else {
            return;
        };

        if Shared::ptr_eq(first, node) {
            return;
        }

        accumulator.push(Diagnostic::duplicate_directive_name(
            name.as_ref().to_string(),
            first.name.span(),
            node.name.span(),
        ));
    }

    fn visit_type_definition(
        &self,
        node: &'ast Shared<'a, T, TypeDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name().ok() else { return };

        let Some(first) = self.0.type_definitions_by_name(name.as_ref()).next() else {
            return;
        };

        if Shared::ptr_eq(first, node) {
            return;
        }

        accumulator.push(Diagnostic::duplicate_type_name(
            name.as_ref().to_string(),
            first.name().span(),
            node.name().span(),
        ));
    }
}
