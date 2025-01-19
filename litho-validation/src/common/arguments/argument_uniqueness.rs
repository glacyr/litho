use std::collections::HashMap;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ArgumentUniqueness<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for ArgumentUniqueness<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_arguments(
        &self,
        node: &'ast Shared<'a, T, Arguments<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut map = HashMap::<&T, &Argument<'a, T>>::new();

        for argument in node.items.iter() {
            match map.get(argument.name.as_ref()) {
                Some(first) => accumulator.push(Diagnostic::duplicate_argument(
                    argument.name.as_ref().to_string(),
                    first.name.span(),
                    argument.name.span(),
                )),
                None => {
                    map.insert(argument.name.as_ref(), argument);
                }
            }
        }
    }
}
