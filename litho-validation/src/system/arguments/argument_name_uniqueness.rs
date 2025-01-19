use std::collections::HashMap;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ArgumentNameUniqueness<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for ArgumentNameUniqueness<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_arguments_definition(
        &self,
        node: &'ast Shared<'a, T, ArgumentsDefinition<'a, T>>,
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
