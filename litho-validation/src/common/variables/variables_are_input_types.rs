use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariablesAreInputTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for VariablesAreInputTypes<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_variable_definition(
        &self,
        node: &'a VariableDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(ty) = node.ty.ok() else {
            return
        };

        let Some(name) = ty.name() else {
            return
        };

        if self.0.type_exists(name) && !self.0.is_input_type(name) {
            accumulator.push(Diagnostic::variable_must_be_input_type(
                node.variable.name.as_ref().to_string(),
                ty.to_string(),
                ty.span(),
            ))
        }
    }
}
