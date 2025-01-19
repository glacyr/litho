use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct DirectivesAreDefined<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for DirectivesAreDefined<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_directive(
        &self,
        node: &'ast Shared<'a, T, Directive<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        if self.0.inference.definition_for_directive(node).is_none() {
            accumulator.push(Diagnostic::undefined_directive(
                name.as_ref().to_string(),
                name.span(),
            ))
        }
    }
}
