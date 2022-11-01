use std::sync::Arc;

use litho_language::ast::*;

use super::Inferred;

#[derive(Debug)]
pub struct Inference<T> {
    pub definition_for_directives: Inferred<Directive<T>, DirectiveDefinition<T>>,
    pub field_definitions_by_field: Inferred<Field<T>, FieldDefinition<T>>,
    pub type_by_selection_set: Inferred<SelectionSet<T>, T>,
    pub definition_for_arguments: Inferred<Arguments<T>, ArgumentsDefinition<T>>,
    pub definitions_for_arguments: Inferred<Argument<T>, InputValueDefinition<T>>,
    pub types_for_values: Inferred<Value<T>, Type<T>>,
    pub default_value_for_values: Inferred<Value<T>, Value<T>>,
}

impl<T> Inference<T> {
    pub fn definition_for_directive(
        &self,
        directive: &Arc<Directive<T>>,
    ) -> Option<&Arc<DirectiveDefinition<T>>> {
        self.definition_for_directives.get(directive)
    }

    pub fn arguments_definition_for_field(
        &self,
        field: &Arc<Field<T>>,
    ) -> Option<&Arc<ArgumentsDefinition<T>>> {
        self.field_definitions_by_field
            .get(field)?
            .arguments_definition
            .as_ref()
    }

    pub fn type_for_field(&self, field: &Arc<Field<T>>) -> Option<&Arc<Type<T>>> {
        self.field_definitions_by_field.get(field)?.ty.ok()
    }
}

impl<T> Default for Inference<T> {
    fn default() -> Self {
        Inference {
            definition_for_directives: Default::default(),
            field_definitions_by_field: Default::default(),
            type_by_selection_set: Default::default(),
            definition_for_arguments: Default::default(),
            definitions_for_arguments: Default::default(),
            types_for_values: Default::default(),
            default_value_for_values: Default::default(),
        }
    }
}
