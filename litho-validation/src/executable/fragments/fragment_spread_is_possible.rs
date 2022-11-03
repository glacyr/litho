use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FragmentSpreadIsPossible<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for FragmentSpreadIsPossible<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_selection_set(
        &self,
        node: &'a Arc<SelectionSet<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(parent_type) = self.0.inference.type_by_selection_set.get(node) else {
            return
        };

        for selection in node.selections.iter() {
            let type_condition = match selection {
                Selection::Field(_) => continue,
                Selection::FragmentSpread(spread) => {
                    match self
                        .0
                        .fragments
                        .by_name(spread.fragment_name.as_ref())
                        .next()
                        .and_then(|def| def.type_condition.ok())
                    {
                        Some(cond) => cond,
                        None => continue,
                    }
                }
                Selection::InlineFragment(fragment) => match fragment.type_condition.as_ref() {
                    Some(cond) => cond,
                    None => continue,
                },
            };

            let Some(fragment_type) = type_condition.named_type.ok() else {
                continue
            };

            if !self.0.type_exists(fragment_type.0.as_ref()) {
                continue;
            }

            let fragment_types = self
                .0
                .possible_types(fragment_type.0.as_ref())
                .collect::<HashSet<&T>>();
            let parent_types = self.0.possible_types(parent_type).collect::<HashSet<&T>>();

            if fragment_types.is_disjoint(&parent_types) {
                accumulator.push(Diagnostic::impossible_fragment_spread(
                    fragment_type.0.as_ref().to_string(),
                    parent_type.to_string(),
                    match selection {
                        Selection::Field(_) => continue,
                        Selection::FragmentSpread(spread) => spread.fragment_name.span(),
                        Selection::InlineFragment(_) => fragment_type.span(),
                    },
                ))
            }
        }
    }
}
