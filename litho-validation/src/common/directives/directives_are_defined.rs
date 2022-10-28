use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct DirectivesAreDefined<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for DirectivesAreDefined<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_directive(&self, node: &'a Arc<Directive<T>>, accumulator: &mut Self::Accumulator) {
        let name = match node.name.ok() {
            Some(name) => name,
            None => return,
        };

        if self.0.inference.definition_for_directive(node).is_none() {
            accumulator.push(Diagnostic::undefined_directive(
                name.as_ref().to_string(),
                name.span(),
            ))
        }
    }
}
