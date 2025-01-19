use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariablesAreInputTypes<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for VariablesAreInputTypes<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_variable_definition(
        &self,
        node: &'ast VariableDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(ty) = node.ty.ok() else { return };

        let Some(name) = ty.name() else { return };

        let Some(var_name) = node.variable.name.ok() else {
            return;
        };

        if self.0.type_exists(name) && !self.0.is_input_type(name) {
            accumulator.push(Diagnostic::variable_must_be_input_type(
                var_name.as_ref().to_string(),
                ty.to_string(),
                ty.span(),
            ))
        }
    }
}
