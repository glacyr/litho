use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct EnumValues<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> EnumValues<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    fn check_enum_values(
        &self,
        name: &T,
        values_definition: &EnumValuesDefinition<'a, T>,
    ) -> Vec<Diagnostic<Span>> {
        let mut errors = vec![];

        for value in values_definition.definitions.iter() {
            match self
                .0
                .enum_value_definitions_by_name(name, value.enum_value.0.as_ref())
                .next()
            {
                Some(first) if !Shared::ptr_eq(first, value) => {
                    errors.push(Diagnostic::duplicate_enum_value(
                        value.enum_value.0.as_ref().to_string(),
                        first.span(),
                        value.enum_value.span(),
                    ));
                    continue;
                }
                Some(_) | None => {}
            }
        }

        errors
    }
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for EnumValues<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_enum_type_definition(
        &self,
        node: &'ast EnumTypeDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        if let Some(values) = node.values_definition.as_ref() {
            accumulator.extend(self.check_enum_values(name.as_ref(), values));
        }

        if self
            .0
            .enum_value_definitions(name.as_ref())
            .next()
            .is_none()
        {
            accumulator.push(Diagnostic::missing_enum_values(
                name.as_ref().to_string(),
                node.values_definition
                    .as_ref()
                    .map(|def| def.span())
                    .unwrap_or(name.span()),
            ));
        }
    }

    fn visit_enum_type_extension(
        &self,
        node: &'ast EnumTypeExtension<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        if let Some(values) = node.values_definition.as_ref() {
            accumulator.extend(self.check_enum_values(name.0.as_ref(), values));
        }
    }
}
