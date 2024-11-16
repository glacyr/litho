use std::collections::HashMap;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariableUniqueness<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for VariableUniqueness<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_variable_definitions(
        &self,
        node: &'a VariableDefinitions<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut map = HashMap::<&T, &Variable<T>>::new();

        for definition in node.variable_definitions.iter() {
            let Some(name) = definition.variable.name.ok() else {
                continue;
            };

            match map.get(name.as_ref()) {
                Some(first) => accumulator.push(Diagnostic::duplicate_variable(
                    name.as_ref().to_string(),
                    first.name.span(),
                    name.span(),
                )),
                None => {
                    map.insert(name.as_ref(), &definition.variable);
                }
            }
        }
    }
}
