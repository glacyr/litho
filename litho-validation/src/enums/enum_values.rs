use std::collections::HashMap;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct EnumValues<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for EnumValues<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_enum_type_definition(
        &self,
        node: &'a EnumTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.name.ok() {
            Some(name) => name,
            None => return,
        };

        let mut visited = HashMap::<&T, &EnumValue<T>>::new();

        for value in node
            .values_definition
            .iter()
            .flat_map(|values| values.definitions.iter())
        {
            match visited.get(&value.enum_value.0.as_ref()) {
                Some(first) => {
                    accumulator.push(Diagnostic::duplicate_enum_value(
                        value.enum_value.0.as_ref().to_string(),
                        first.span(),
                        value.enum_value.span(),
                    ));
                    continue;
                }
                None => {}
            }

            visited.insert(value.enum_value.0.as_ref(), &value.enum_value);
        }

        if visited.is_empty() {
            accumulator.push(Diagnostic::missing_enum_values(
                name.as_ref().to_string(),
                node.values_definition
                    .as_ref()
                    .map(|def| def.span())
                    .unwrap_or(name.span()),
            ));
        }
    }
}
