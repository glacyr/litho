use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ArgumentUniqueness<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ArgumentUniqueness<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_arguments(&self, node: &'a Arc<Arguments<T>>, accumulator: &mut Self::Accumulator) {
        let mut map = HashMap::<&T, &Argument<T>>::new();

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
