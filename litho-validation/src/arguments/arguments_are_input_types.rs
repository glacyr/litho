use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct ArgumentsAreInputTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ArgumentsAreInputTypes<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_input_value_definition(
        &self,
        node: &'a Arc<InputValueDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.ty.ok().and_then(|ty| ty.named_type()) {
            Some(name) if !self.0.is_input_type(name.0.as_ref()) => {
                accumulator.push(Diagnostic::input_value_not_input_type(
                    node.name.as_ref().to_string(),
                    name.0.as_ref().to_string(),
                    name.span(),
                ));
            }
            _ => {}
        }
    }
}
