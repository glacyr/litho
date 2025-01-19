use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentNameUniqueness<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for FragmentNameUniqueness<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_fragment_definition(
        &self,
        node: &'ast Shared<'a, T, FragmentDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.fragment_name.ok() else {
            return;
        };

        match self.0.fragments.by_name(name.as_ref()).next() {
            Some(first) if !Shared::ptr_eq(first, node) => {
                accumulator.push(Diagnostic::duplicate_fragment_name(
                    name.as_ref().to_string(),
                    first.fragment_name.span(),
                    node.fragment_name.span(),
                ))
            }
            _ => {}
        }
    }
}
