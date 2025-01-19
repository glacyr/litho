use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentsMustBeUsed<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for FragmentsMustBeUsed<'ast, 'a, T>
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

        if self.0.usages.fragments.usages(node).next().is_none() {
            accumulator.push(Diagnostic::unused_fragment_definition(
                name.as_ref().to_string(),
                name.span(),
            ));
        }
    }
}
