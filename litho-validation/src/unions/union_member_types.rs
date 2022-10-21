use std::collections::HashMap;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct UnionMemberTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for UnionMemberTypes<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_union_type_definition(
        &self,
        node: &'a UnionTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.name.ok() {
            Some(name) => name,
            None => return,
        };

        let mut visited = HashMap::<&T, &NamedType<T>>::new();

        for ty in node
            .member_types
            .iter()
            .flat_map(|types| types.named_types())
        {
            match visited.get(&ty.0.as_ref()) {
                Some(first) => {
                    accumulator.push(Diagnostic::duplicate_union_member(
                        ty.0.as_ref().to_string(),
                        first.span(),
                        ty.span(),
                    ));
                    continue;
                }
                None => {}
            }

            visited.insert(ty.0.as_ref(), ty);

            if !self.0.is_object_type(ty.0.as_ref()) {
                accumulator.push(Diagnostic::non_object_union_member(
                    ty.0.as_ref().to_string(),
                    ty.span(),
                ));
            }
        }

        if visited.is_empty() {
            accumulator.push(Diagnostic::missing_union_members(
                name.as_ref().to_string(),
                name.span(),
            ));
        }
    }
}
