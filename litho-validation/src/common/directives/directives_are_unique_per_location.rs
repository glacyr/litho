use std::collections::HashMap;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct DirectivesAreUniquePerLocation<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for DirectivesAreUniquePerLocation<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_directives(&self, node: &'a Directives<T>, accumulator: &mut Self::Accumulator) {
        let mut seen = HashMap::<&T, &Directive<T>>::new();

        for directive in node.directives.iter() {
            let Some(definition) = self.0.inference.definition_for_directive(directive) else {
                return
            };

            if definition.repeatable.is_some() {
                continue;
            }

            let Some(name) = definition.name.ok() else {
                continue
            };

            match seen.get(name.as_ref()) {
                Some(first) => accumulator.push(Diagnostic::duplicate_non_repeatable_directive(
                    name.as_ref().to_string(),
                    first.name.span(),
                    directive.name.span(),
                )),
                None => {
                    seen.insert(name.as_ref(), directive);
                }
            }
        }
    }
}
