use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct HasFields<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> HasFields<'a, T>
where
    T: Eq + Hash + ToString,
{
    pub fn check_fields_definition(
        &self,
        name: &'a Name<T>,
        definition: Option<&FieldsDefinition<T>>,
    ) -> Option<Diagnostic<Span>> {
        match self.0.field_definitions_by_type(name.as_ref()).next() {
            Some(_) => None,
            None => Some(Diagnostic::empty_type(
                name.as_ref().to_string(),
                definition
                    .as_ref()
                    .map(|def| def.braces.span())
                    .unwrap_or(name.span()),
            )),
        }
    }
}

impl<'a, T> Visit<'a, T> for HasFields<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_interface_type_definition(
        &self,
        node: &'a InterfaceTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            accumulator.extend(
                self.check_fields_definition(name, node.fields_definition.as_ref())
                    .into_iter(),
            )
        }
    }

    fn visit_object_type_definition(
        &self,
        node: &'a ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            accumulator.extend(
                self.check_fields_definition(name, node.fields_definition.as_ref())
                    .into_iter(),
            );
        }
    }
}
