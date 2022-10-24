use litho_language::ast::*;

use super::Inferred;

#[derive(Debug)]
pub struct Inference<T> {
    pub field_definitions_by_field: Inferred<Field<T>, FieldDefinition<T>>,
    pub type_by_selection_set: Inferred<SelectionSet<T>, T>,
    pub definition_for_arguments: Inferred<Arguments<T>, ArgumentsDefinition<T>>,
    pub types_for_values: Inferred<Value<T>, Type<T>>,
}

impl<T> Default for Inference<T> {
    fn default() -> Self {
        Inference {
            field_definitions_by_field: Default::default(),
            type_by_selection_set: Default::default(),
            definition_for_arguments: Default::default(),
            types_for_values: Default::default(),
        }
    }
}
