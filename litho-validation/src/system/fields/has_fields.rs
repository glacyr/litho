use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct HasFields<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> HasFields<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    pub fn check_fields_definition(
        &self,
        name: &'ast Name<'a, T>,
        definition: Option<&FieldsDefinition<'a, T>>,
    ) -> Option<Diagnostic<Span>> {
        match self.0.field_definitions(name.as_ref()).next() {
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

impl<'ast, 'a, T> Visit<'ast, 'a, T> for HasFields<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_interface_type_definition(
        &self,
        node: &'ast InterfaceTypeDefinition<'a, T>,
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
        node: &'ast ObjectTypeDefinition<'a, T>,
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
