use std::collections::HashMap;
use std::hash::Hash;

use litho_language::ast::{NamedType, Node, Visit};
use litho_types::Database;

use crate::Error;

pub struct UnionMemberTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for UnionMemberTypes<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_union_type_definition(
        &self,
        node: &'a litho_language::ast::UnionTypeDefinition<T>,
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
                    accumulator.push(Error::DuplicateUnionMember {
                        name: ty.0.as_ref(),
                        first: first.span(),
                        second: ty.span(),
                    });
                    continue;
                }
                None => {}
            }

            visited.insert(ty.0.as_ref(), ty);

            if !self.0.is_object_type(ty.0.as_ref()) {
                accumulator.push(Error::NonObjectUnionMember {
                    name: ty.0.as_ref(),
                    span: ty.span(),
                })
            }
        }

        if visited.is_empty() {
            accumulator.push(Error::MissingUnionMembers {
                name: name.as_ref(),
                span: name.span(),
            })
        }
    }
}
