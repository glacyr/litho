use std::collections::HashMap;
use std::hash::Hash;
use std::iter::once;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct ObjectImplementsInterfaces<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for ObjectImplementsInterfaces<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_object_type_definition(
        &self,
        node: &'a ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let interfaces = match node.implements_interfaces.as_ref() {
            Some(interfaces) => interfaces,
            None => return,
        };

        let mut visited = HashMap::<&T, &NamedType<T>>::new();

        for interface in
            once(&interfaces.first).chain(interfaces.types.iter().map(|(_, interface)| interface))
        {
            let interface = match interface.ok() {
                Some(interface) => interface,
                _ => continue,
            };

            match visited.get(&interface.0.as_ref()) {
                Some(exists) => {
                    accumulator.push(Error::DuplicateImplementsInterface {
                        name: interface.0.as_ref(),
                        first: exists.span(),
                        second: interface.span(),
                    });
                    continue;
                }
                None => {}
            }

            visited.insert(interface.0.as_ref(), interface);
        }
    }
}
