use litho_language::ast::*;

use super::{Inferred, InferredMany, InferredSimple};

#[derive(Debug)]
pub struct Inference<'a, T>
where
    T: ContextValue<'a>,
{
    pub definition_for_directives: Inferred<'a, T, Directive<'a, T>, DirectiveDefinition<'a, T>>,
    pub field_definitions_by_field: Inferred<'a, T, Field<'a, T>, FieldDefinition<'a, T>>,
    pub type_by_selection_set: InferredSimple<'a, T, SelectionSet<'a, T>, T>,
    pub definition_for_arguments: Inferred<'a, T, Arguments<'a, T>, ArgumentsDefinition<'a, T>>,
    pub definitions_for_arguments: Inferred<'a, T, Argument<'a, T>, InputValueDefinition<'a, T>>,
    pub types_for_values: Inferred<'a, T, Value<'a, T>, Type<'a, T>>,
    pub default_value_for_values: Inferred<'a, T, Value<'a, T>, Value<'a, T>>,
    pub definitions_for_variable: InferredMany<'a, T, Value<'a, T>, VariableDefinition<'a, T>>,
}

impl<'a, T> Inference<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn definition_for_directive(
        &self,
        directive: &Shared<'a, T, Directive<'a, T>>,
    ) -> Option<&Shared<'a, T, DirectiveDefinition<'a, T>>> {
        self.definition_for_directives.get(directive)
    }

    pub fn arguments_definition_for_field(
        &self,
        field: &Shared<'a, T, Field<'a, T>>,
    ) -> Option<&Shared<'a, T, ArgumentsDefinition<'a, T>>> {
        self.field_definitions_by_field
            .get(field)?
            .arguments_definition
            .as_ref()
    }

    pub fn type_for_field(
        &self,
        field: &Shared<'a, T, Field<'a, T>>,
    ) -> Option<&Shared<'a, T, Type<'a, T>>> {
        self.field_definitions_by_field.get(field)?.ty.ok()
    }
}

impl<'a, T> Default for Inference<'a, T>
where
    T: ContextValue<'a>,
{
    fn default() -> Self {
        Inference {
            definition_for_directives: Default::default(),
            field_definitions_by_field: Default::default(),
            type_by_selection_set: Default::default(),
            definition_for_arguments: Default::default(),
            definitions_for_arguments: Default::default(),
            types_for_values: Default::default(),
            default_value_for_values: Default::default(),
            definitions_for_variable: Default::default(),
        }
    }
}
