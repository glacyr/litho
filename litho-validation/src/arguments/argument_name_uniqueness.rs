use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ArgumentNameUniqueness<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ArgumentNameUniqueness<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_arguments_definition(
        &self,
        node: &'a Arc<ArgumentsDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut existing = HashMap::<&T, &InputValueDefinition<T>>::new();

        for field in node.definitions.iter() {
            match existing.get(&field.name.as_ref()) {
                Some(first) => accumulator.push(Diagnostic::duplicate_argument_name(
                    field.name.as_ref().to_string(),
                    first.name.span(),
                    field.name.span(),
                )),
                None => {
                    existing.insert(field.name.as_ref(), field);
                }
            }
        }
    }
}
