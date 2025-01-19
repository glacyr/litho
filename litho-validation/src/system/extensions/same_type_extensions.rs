use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct SameTypeExtensions<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> Visit<'ast, 'a, T> for SameTypeExtensions<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_type_extension(
        &self,
        node: &'ast Shared<'a, T, TypeExtension<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name() else { return };

        let Some(first) = self
            .0
            .type_definitions_by_name(name)
            .next()
            .map(|first| first.keyword())
        else {
            return;
        };

        let second = node.keyword();

        if first.as_ref() != second.as_ref() {
            accumulator.push(Diagnostic::different_extension_type(
                name.to_string(),
                first.as_ref().to_string(),
                second.as_ref().to_string(),
                first.span(),
                second.span(),
            ));
        }
    }
}
