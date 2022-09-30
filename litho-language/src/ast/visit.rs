use crate::lex::{Span, Token};

use super::types::*;

macro_rules! visit {
    ($name:ident, $ty:ident) => {
        fn $name(&self, node: &'ast $ty<'a>, accumulator: &mut Self::Accumulator) {}
    };
}

#[allow(unused_variables)]
pub trait Visit<'ast, 'a> {
    type Accumulator;

    fn visit_recoverable<T>(
        &self,
        node: &'ast Recoverable<T>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_span(&self, span: Span, accumulator: &mut Self::Accumulator) {}

    visit!(visit_document, Document);
    visit!(visit_definition, Definition);
    visit!(visit_executable_document, ExecutableDocument);
    visit!(visit_executable_definition, ExecutableDefinition);
    visit!(visit_operation_definition, OperationDefinition);
    visit!(visit_operation_type, OperationType);
    visit!(visit_selection_set, SelectionSet);
    visit!(visit_selection, Selection);
    visit!(visit_field, Field);
    visit!(visit_alias, Alias);
    visit!(visit_arguments, Arguments);
    visit!(visit_argument, Argument);
    visit!(visit_fragment_spread, FragmentSpread);
    visit!(visit_inline_fragment, InlineFragment);
    visit!(visit_fragment_definition, FragmentDefinition);
    visit!(visit_type_condition, TypeCondition);
    visit!(visit_value, Value);
    visit!(visit_boolean_value, BooleanValue);
    visit!(visit_null_value, NullValue);
    visit!(visit_enum_value, EnumValue);
    visit!(visit_list_value, ListValue);
    visit!(visit_object_value, ObjectValue);
    visit!(visit_object_field, ObjectField);
    visit!(visit_variable_definitions, VariableDefinitions);
    visit!(visit_variable_definition, VariableDefinition);
    visit!(visit_variable, Variable);
    visit!(visit_default_value, DefaultValue);
    visit!(visit_type, Type);
    visit!(visit_named_type, NamedType);
    visit!(visit_list_type, ListType);
    visit!(visit_non_null_type, NonNullType);
    visit!(visit_directives, Directives);
    visit!(visit_directive, Directive);
}
