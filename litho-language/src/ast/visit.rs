use std::sync::Arc;

use crate::lex::{FloatValue, IntValue, Span, StringValue};

use super::types::*;

macro_rules! visit {
    ($name:ident, Arc<$ty:ident>) => {
        fn $name(&self, node: &'ast Arc<$ty<T>>, accumulator: &mut Self::Accumulator) {}
    };

    ($name:ident, $ty:ident) => {
        fn $name(&self, node: &'ast $ty<T>, accumulator: &mut Self::Accumulator) {}
    };
}

#[allow(unused_variables)]
pub trait Visit<'ast, T> {
    type Accumulator;

    fn visit_recoverable<U>(
        &self,
        node: &'ast Recoverable<U>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_span(&self, span: Span, accumulator: &mut Self::Accumulator) {}

    visit!(visit_document, Document);
    visit!(visit_definition, Definition);
    visit!(visit_executable_document, ExecutableDocument);
    visit!(visit_executable_definition, ExecutableDefinition);
    visit!(visit_operation_definition, Arc<OperationDefinition>);
    visit!(post_visit_operation_definition, Arc<OperationDefinition>);
    visit!(visit_operation_type, OperationType);
    visit!(visit_selection_set, Arc<SelectionSet>);
    visit!(visit_selection, Selection);
    visit!(visit_field, Arc<Field>);
    visit!(post_visit_field, Arc<Field>);
    visit!(visit_alias, Alias);
    visit!(visit_arguments, Arc<Arguments>);
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
    visit!(visit_type_system_document, TypeSystemDocument);
    visit!(visit_type_system_definition, TypeSystemDefinition);
    visit!(
        visit_type_system_extension_document,
        TypeSystemExtensionDocument
    );
    visit!(
        visit_type_system_definition_or_extension,
        TypeSystemDefinitionOrExtension
    );
    visit!(visit_type_system_extension, TypeSystemExtension);
    visit!(visit_description, Description);
    visit!(visit_schema_definition, SchemaDefinition);
    visit!(
        visit_root_operation_type_definitions,
        RootOperationTypeDefinitions
    );
    visit!(
        visit_root_operation_type_definition,
        RootOperationTypeDefinition
    );
    visit!(visit_schema_extension, SchemaExtension);
    visit!(visit_type_definition, Arc<TypeDefinition>);
    visit!(visit_type_extension, Arc<TypeExtension>);
    visit!(visit_scalar_type_definition, ScalarTypeDefinition);
    visit!(visit_scalar_type_extension, ScalarTypeExtension);
    visit!(visit_object_type_definition, ObjectTypeDefinition);
    visit!(visit_implements_interfaces, ImplementsInterfaces);
    visit!(visit_fields_definition, FieldsDefinition);
    visit!(visit_field_definition, Arc<FieldDefinition>);
    visit!(visit_arguments_definition, Arc<ArgumentsDefinition>);
    visit!(visit_input_value_definition, InputValueDefinition);
    visit!(visit_object_type_extension, ObjectTypeExtension);
    visit!(visit_interface_type_definition, InterfaceTypeDefinition);
    visit!(visit_interface_type_extension, InterfaceTypeExtension);
    visit!(visit_union_type_definition, UnionTypeDefinition);
    visit!(visit_union_member_types, UnionMemberTypes);
    visit!(visit_union_type_extension, UnionTypeExtension);
    visit!(visit_enum_type_definition, EnumTypeDefinition);
    visit!(visit_enum_values_definition, EnumValuesDefinition);
    visit!(visit_enum_value_definition, EnumValueDefinition);
    visit!(visit_enum_type_extension, EnumTypeExtension);
    visit!(
        visit_input_object_type_definition,
        InputObjectTypeDefinition
    );
    visit!(visit_input_fields_definition, InputFieldsDefinition);
    visit!(visit_input_object_type_extension, InputObjectTypeExtension);
    visit!(visit_directive_definition, Arc<DirectiveDefinition>);
    visit!(visit_directive_locations, DirectiveLocations);
    visit!(visit_directive_location, DirectiveLocation);
    visit!(
        visit_executable_directive_location,
        ExecutableDirectiveLocation
    );
    visit!(
        visit_type_system_directive_location,
        TypeSystemDirectiveLocation
    );

    visit!(visit_int_value, IntValue);
    visit!(visit_float_value, FloatValue);
    visit!(visit_string_value, StringValue);
}
