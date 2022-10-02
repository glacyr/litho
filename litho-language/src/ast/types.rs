use crate::lex::{FloatValue, IntValue, Name, Punctuator, Span, StringValue};

use super::{node, node_enum, node_unit, Node, Visit};

#[derive(Debug, Clone)]
pub enum Missing {
    Unknown,
    Simple(&'static str, &'static str),
    Delimiter(&'static str, &'static str, Span, &'static str),
}

impl Missing {
    pub fn delimiter_complaint<'a, T>(
        message: &'static str,
        first_label: &'static str,
        second_label: &'static str,
    ) -> impl Fn(&T) -> Missing
    where
        T: Node<'a>,
    {
        move |left| Missing::Delimiter(message, first_label, left.span(), second_label)
    }
}

impl Default for Missing {
    fn default() -> Self {
        Missing::Unknown
    }
}

impl From<&'static str> for Missing {
    fn from(title: &'static str) -> Self {
        Missing::Simple(title, title)
    }
}

impl wrom::Missing for Missing {
    type Error = MissingToken;
}

#[derive(Debug, Clone)]
pub struct MissingToken {
    pub span: Span,
    pub missing: Missing,
}

pub type Recoverable<T> = wrom::Recoverable<T, Missing>;

/// # 2.2
/// A GraphQL Document describes a complete file or request string operated on
/// by a GraphQL service or client. A document contains multiple definitions,
/// either executable or representative of a GraphQL type system.
#[derive(Clone, Debug, Default)]
pub struct Document<'a> {
    pub definitions: Vec<Definition<'a>>,
}

node!(Document, visit_document, definitions);

#[derive(Clone, Debug)]
pub enum Definition<'a> {
    ExecutableDefinition(ExecutableDefinition<'a>),
    TypeSystemDefinitionOrExtension(TypeSystemDefinitionOrExtension<'a>),
}

node_enum!(
    Definition,
    visit_definition,
    ExecutableDefinition,
    TypeSystemDefinitionOrExtension
);

/// Documents are only executable by a GraphQL service if they are
/// `ExecutableDocument` and contain at least one `OperationDefinition`. A
/// Document which contains `TypeSystemDefinitionOrExtension` must not be
/// executed; GraphQL execution services which receive a Document containing
/// these should return a descriptive error.
#[derive(Clone, Debug)]
pub struct ExecutableDocument<'a> {
    pub definitions: Vec<ExecutableDefinition<'a>>,
}

node!(ExecutableDocument, visit_executable_document, definitions);

#[derive(Clone, Debug)]
pub enum ExecutableDefinition<'a> {
    OperationDefinition(OperationDefinition<'a>),
    FragmentDefinition(FragmentDefinition<'a>),
}

node_enum!(
    ExecutableDefinition,
    visit_executable_definition,
    OperationDefinition,
    FragmentDefinition
);

#[derive(Clone, Debug)]
pub struct OperationDefinition<'a> {
    pub ty: Option<OperationType<'a>>,
    pub name: Recoverable<Name<'a>>,
    pub variable_definitions: Option<VariableDefinitions<'a>>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: Recoverable<SelectionSet<'a>>,
}

node!(
    OperationDefinition,
    visit_operation_definition,
    ty,
    name,
    variable_definitions,
    directives,
    selection_set
);

#[derive(Clone, Copy, Debug)]
pub enum OperationType<'a> {
    Query(Name<'a>),
    Mutation(Name<'a>),
    Subscription(Name<'a>),
}

node_enum!(
    OperationType,
    visit_operation_type,
    Query,
    Mutation,
    Subscription
);

#[derive(Clone, Debug)]
pub struct SelectionSet<'a> {
    pub braces: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub selections: Vec<Selection<'a>>,
}

node!(SelectionSet, visit_selection_set, braces, selections);

#[derive(Clone, Debug)]
pub enum Selection<'a> {
    Field(Field<'a>),
    FragmentSpread(FragmentSpread<'a>),
    InlineFragment(InlineFragment<'a>),
}

node_enum!(
    Selection,
    visit_selection,
    Field,
    FragmentSpread,
    InlineFragment
);

#[derive(Clone, Debug)]
pub struct Field<'a> {
    pub alias: Option<Alias<'a>>,
    pub name: Recoverable<Name<'a>>,
    pub arguments: Option<Arguments<'a>>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: Option<SelectionSet<'a>>,
}

node!(
    Field,
    visit_field,
    alias,
    name,
    arguments,
    directives,
    selection_set
);

#[derive(Clone, Debug)]
pub struct Alias<'a> {
    pub name: Name<'a>,
    pub colon: Punctuator<'a>,
}

node!(Alias, visit_alias, name, colon);

#[derive(Clone, Debug)]
pub struct Arguments<'a> {
    pub parens: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub items: Vec<Argument<'a>>,
}

node!(Arguments, visit_arguments, parens, items);

#[derive(Clone, Debug)]
pub struct Argument<'a> {
    pub name: Name<'a>,
    pub colon: Recoverable<Punctuator<'a>>,
    pub value: Recoverable<Value<'a>>,
}

node!(Argument, visit_argument, name, colon, value);

#[derive(Clone, Debug)]
pub struct FragmentSpread<'a> {
    pub dots: Punctuator<'a>,
    pub fragment_name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
}

node!(
    FragmentSpread,
    visit_fragment_spread,
    dots,
    fragment_name,
    directives
);

#[derive(Clone, Debug)]
pub struct InlineFragment<'a> {
    pub dots: Punctuator<'a>,
    pub type_condition: Option<TypeCondition<'a>>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: Recoverable<SelectionSet<'a>>,
}

node!(
    InlineFragment,
    visit_inline_fragment,
    dots,
    type_condition,
    directives,
    selection_set
);

#[derive(Clone, Debug)]
pub struct FragmentDefinition<'a> {
    pub fragment: Name<'a>,
    pub fragment_name: Recoverable<Name<'a>>,
    pub type_condition: Recoverable<TypeCondition<'a>>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: Recoverable<SelectionSet<'a>>,
}

node!(
    FragmentDefinition,
    visit_fragment_definition,
    fragment,
    fragment_name,
    type_condition,
    directives,
    selection_set
);

#[derive(Clone, Debug)]
pub struct TypeCondition<'a> {
    pub on: Name<'a>,
    pub named_type: Recoverable<Name<'a>>,
}

node!(TypeCondition, visit_type_condition, on, named_type);

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Variable(Variable<'a>),
    IntValue(IntValue<'a>),
    FloatValue(FloatValue<'a>),
    StringValue(StringValue<'a>),
    BooleanValue(BooleanValue<'a>),
    NullValue(NullValue<'a>),
    EnumValue(EnumValue<'a>),
    ListValue(ListValue<'a>),
    ObjectValue(ObjectValue<'a>),
}

node_enum!(
    Value,
    visit_value,
    Variable,
    IntValue,
    FloatValue,
    StringValue,
    BooleanValue,
    NullValue,
    EnumValue,
    ListValue,
    ObjectValue,
);

#[derive(Clone, Debug)]
pub enum BooleanValue<'a> {
    True(Name<'a>),
    False(Name<'a>),
}

node_enum!(BooleanValue, visit_boolean_value, True, False);

#[derive(Clone, Debug)]
pub struct NullValue<'a>(pub Name<'a>);

node_unit!(NullValue, visit_null_value);

#[derive(Clone, Debug)]
pub struct EnumValue<'a>(pub Name<'a>);

node_unit!(EnumValue, visit_enum_value);

#[derive(Clone, Debug)]
pub struct ListValue<'a> {
    pub brackets: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub values: Vec<Value<'a>>,
}

node!(ListValue, visit_list_value, brackets, values);

#[derive(Clone, Debug)]
pub struct ObjectValue<'a> {
    pub braces: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub object_fields: Vec<ObjectField<'a>>,
}

node!(ObjectValue, visit_object_value, braces, object_fields);

#[derive(Clone, Debug)]
pub struct ObjectField<'a> {
    pub name: Name<'a>,
    pub colon: Recoverable<Punctuator<'a>>,
    pub value: Recoverable<Value<'a>>,
}

node!(ObjectField, visit_object_field, name, colon, value);

#[derive(Clone, Debug)]
pub struct VariableDefinitions<'a> {
    pub parens: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub variable_definitions: Vec<VariableDefinition<'a>>,
}

node!(
    VariableDefinitions,
    visit_variable_definitions,
    parens,
    variable_definitions
);

#[derive(Clone, Debug)]
pub struct VariableDefinition<'a> {
    pub variable: Variable<'a>,
    pub colon: Recoverable<Punctuator<'a>>,
    pub ty: Recoverable<Type<'a>>,
    pub default_value: Option<DefaultValue<'a>>,
    pub directives: Option<Directives<'a>>,
}

node!(
    VariableDefinition,
    visit_variable_definition,
    variable,
    colon,
    ty,
    default_value,
    directives
);

#[derive(Clone, Debug)]
pub struct Variable<'a> {
    pub dollar: Punctuator<'a>,
    pub name: Name<'a>,
}

node!(Variable, visit_variable, dollar, name);

#[derive(Clone, Debug)]
pub struct DefaultValue<'a> {
    pub eq: Punctuator<'a>,
    pub value: Recoverable<Value<'a>>,
}

node!(DefaultValue, visit_default_value, eq, value);

#[derive(Clone, Debug)]
pub enum Type<'a> {
    Named(NamedType<'a>),
    List(Box<ListType<'a>>),
    NonNull(Box<NonNullType<'a>>),
}

node_enum!(Type, visit_type, Named, List, NonNull);

#[derive(Clone, Debug)]
pub struct NamedType<'a>(pub Name<'a>);

node_unit!(NamedType, visit_named_type);

#[derive(Clone, Debug)]
pub struct ListType<'a> {
    pub brackets: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub ty: Recoverable<Type<'a>>,
}

node!(ListType, visit_list_type, brackets, ty);

#[derive(Clone, Debug)]
pub struct NonNullType<'a> {
    pub ty: Type<'a>,
    pub bang: Punctuator<'a>,
}

node!(NonNullType, visit_non_null_type, ty, bang);

#[derive(Clone, Debug)]
pub struct Directives<'a> {
    pub directives: Vec<Directive<'a>>,
}

node!(Directives, visit_directives, directives);

#[derive(Clone, Debug)]
pub struct Directive<'a> {
    pub at: Punctuator<'a>,
    pub name: Recoverable<Name<'a>>,
    pub arguments: Option<Arguments<'a>>,
}

node!(Directive, visit_directive, at, name, arguments);

/// The GraphQL Type system describes the capabilities of a GraphQL service and
/// is used to determine if a requested operation is valid, to guarantee the
/// type of response results, and describes the input types of variables to
/// determine if values provided at request time are valid.
///
/// The GraphQL language includes an IDL used to describe a GraphQL service's
/// type system. Tools may use this definition language to provide utilities
/// such as client code generation or service bootstrapping.
///
/// GraphQL tools or services which only seek to execute GraphQL requests and
/// not construct a new GraphQL schema may choose not to allow
/// `TypeSystemDefinition`. Tools which only seek to produce schema and not
/// execute requests may choose to only allow `TypeSystemDocument` and not allow
/// `ExecutableDefinition` or `TypeSystemExtension` but should provide a
/// descriptive error if present.
#[derive(Clone, Debug)]
pub struct TypeSystemDocument<'a> {
    pub definitions: Vec<TypeSystemDefinition<'a>>,
}

node!(TypeSystemDocument, visit_type_system_document, definitions);

#[derive(Clone, Debug)]
pub enum TypeSystemDefinition<'a> {
    SchemaDefinition(SchemaDefinition<'a>),
    TypeDefinition(TypeDefinition<'a>),
    DirectiveDefinition(DirectiveDefinition<'a>),
}

node_enum!(
    TypeSystemDefinition,
    visit_type_system_definition,
    SchemaDefinition,
    TypeDefinition,
    DirectiveDefinition
);

#[derive(Clone, Debug)]
pub struct TypeSystemExtensionDocument<'a> {
    pub definitions: Vec<TypeSystemDefinitionOrExtension<'a>>,
}

node!(
    TypeSystemExtensionDocument,
    visit_type_system_extension_document,
    definitions
);

#[derive(Clone, Debug)]
pub enum TypeSystemDefinitionOrExtension<'a> {
    TypeSystemDefinition(TypeSystemDefinition<'a>),
    TypeSystemExtension(TypeSystemExtension<'a>),
}

node_enum!(
    TypeSystemDefinitionOrExtension,
    visit_type_system_definition_or_extension,
    TypeSystemDefinition,
    TypeSystemExtension
);

#[derive(Clone, Debug)]
pub enum TypeSystemExtension<'a> {
    SchemaExtension(SchemaExtension<'a>),
    TypeExtension(TypeExtension<'a>),
}

node_enum!(
    TypeSystemExtension,
    visit_type_system_extension,
    SchemaExtension,
    TypeExtension
);

#[derive(Clone, Debug)]
pub struct Description<'a>(pub StringValue<'a>);

impl Description<'_> {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

node_unit!(Description, visit_description);

#[derive(Clone, Debug)]
pub struct SchemaDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub schema: Name<'a>,
    pub directives: Option<Directives<'a>>,
    pub type_definitions: Recoverable<RootOperationTypeDefinitions<'a>>,
}

node!(
    SchemaDefinition,
    visit_schema_definition,
    description,
    schema,
    directives,
    type_definitions
);

#[derive(Clone, Debug)]
pub struct RootOperationTypeDefinitions<'a> {
    pub braces: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub definitions: Vec<RootOperationTypeDefinition<'a>>,
}

node!(
    RootOperationTypeDefinitions,
    visit_root_operation_type_definitions,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct RootOperationTypeDefinition<'a> {
    pub operation_type: OperationType<'a>,
    pub colon: Recoverable<Punctuator<'a>>,
    pub named_type: Recoverable<NamedType<'a>>,
}

node!(
    RootOperationTypeDefinition,
    visit_root_operation_type_definition,
    operation_type,
    colon,
    named_type
);

#[derive(Clone, Debug)]
pub struct SchemaExtension<'a> {
    pub extend_schema: (Name<'a>, Name<'a>),
    pub directives: Option<Directives<'a>>,
    pub type_definitions: Option<RootOperationTypeDefinitions<'a>>,
}

node!(
    SchemaExtension,
    visit_schema_extension,
    extend_schema,
    directives,
    type_definitions
);

#[derive(Clone, Debug)]
pub enum TypeDefinition<'a> {
    ScalarTypeDefinition(ScalarTypeDefinition<'a>),
    ObjectTypeDefinition(ObjectTypeDefinition<'a>),
    InterfaceTypeDefinition(InterfaceTypeDefinition<'a>),
    UnionTypeDefinition(UnionTypeDefinition<'a>),
    EnumTypeDefinition(EnumTypeDefinition<'a>),
    InputObjectTypeDefinition(InputObjectTypeDefinition<'a>),
}

node_enum!(
    TypeDefinition,
    visit_type_definition,
    ScalarTypeDefinition,
    ObjectTypeDefinition,
    InterfaceTypeDefinition,
    UnionTypeDefinition,
    EnumTypeDefinition,
    InputObjectTypeDefinition
);

#[derive(Clone, Debug)]
pub enum TypeExtension<'a> {
    ScalarTypeExtension(ScalarTypeExtension<'a>),
    ObjectTypeExtension(ObjectTypeExtension<'a>),
    InterfaceTypeExtension(InterfaceTypeExtension<'a>),
    UnionTypeExtension(UnionTypeExtension<'a>),
    EnumTypeExtension(EnumTypeExtension<'a>),
    InputObjectTypeExtension(InputObjectTypeExtension<'a>),
}

node_enum!(
    TypeExtension,
    visit_type_extension,
    ScalarTypeExtension,
    ObjectTypeExtension,
    InterfaceTypeExtension,
    UnionTypeExtension,
    EnumTypeExtension,
    InputObjectTypeExtension
);

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub scalar: Name<'a>,
    pub name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
}

node!(
    ScalarTypeDefinition,
    visit_scalar_type_definition,
    scalar,
    name,
    directives
);

#[derive(Clone, Debug)]
pub struct ScalarTypeExtension<'a> {
    pub extend_scalar: (Name<'a>, Name<'a>),
    pub name: Recoverable<Name<'a>>,
    pub directives: Recoverable<Directives<'a>>,
}

node!(
    ScalarTypeExtension,
    visit_scalar_type_extension,
    extend_scalar,
    name,
    directives
);

#[derive(Clone, Debug)]
pub struct ObjectTypeDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub ty: Name<'a>,
    pub name: Recoverable<Name<'a>>,
    pub implements_interfaces: Option<ImplementsInterfaces<'a>>,
    pub directives: Option<Directives<'a>>,
    pub fields_definition: Option<FieldsDefinition<'a>>,
}

node!(
    ObjectTypeDefinition,
    visit_object_type_definition,
    description,
    ty,
    name,
    implements_interfaces,
    directives,
    fields_definition
);

#[derive(Clone, Debug)]
pub struct ImplementsInterfaces<'a> {
    pub implements: Name<'a>,
    pub ampersand: Option<Punctuator<'a>>,
    pub first: Recoverable<NamedType<'a>>,
    pub types: Vec<(Punctuator<'a>, Recoverable<NamedType<'a>>)>,
}

node!(
    ImplementsInterfaces,
    visit_implements_interfaces,
    implements,
    ampersand,
    first,
    types
);

#[derive(Clone, Debug)]
pub struct FieldsDefinition<'a> {
    pub braces: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub definitions: Vec<FieldDefinition<'a>>,
}

node!(
    FieldsDefinition,
    visit_fields_definition,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct FieldDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub name: Name<'a>,
    pub arguments_definition: Option<ArgumentsDefinition<'a>>,
    pub colon: Recoverable<Punctuator<'a>>,
    pub ty: Recoverable<Type<'a>>,
    pub directives: Option<Directives<'a>>,
}

node!(
    FieldDefinition,
    visit_field_definition,
    description,
    name,
    arguments_definition,
    colon,
    ty,
    directives
);

#[derive(Clone, Debug)]
pub struct ArgumentsDefinition<'a> {
    pub parens: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub definitions: Vec<InputValueDefinition<'a>>,
}

node!(
    ArgumentsDefinition,
    visit_arguments_definition,
    parens,
    definitions
);

#[derive(Clone, Debug)]
pub struct InputValueDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub name: Name<'a>,
    pub colon: Recoverable<Punctuator<'a>>,
    pub ty: Recoverable<Type<'a>>,
    pub default_value: Option<DefaultValue<'a>>,
    pub directives: Option<Directives<'a>>,
}

node!(
    InputValueDefinition,
    visit_input_value_definition,
    description,
    name,
    colon,
    ty,
    default_value,
    directives
);

#[derive(Clone, Debug)]
pub struct ObjectTypeExtension<'a> {
    pub extend_type: (Name<'a>, Name<'a>),
    pub name: Recoverable<Name<'a>>,
    pub implements_interfaces: Option<ImplementsInterfaces<'a>>,
    pub directives: Option<Directives<'a>>,
    pub fields_definition: Option<FieldsDefinition<'a>>,
}

node!(
    ObjectTypeExtension,
    visit_object_type_extension,
    extend_type,
    name,
    implements_interfaces,
    directives,
    fields_definition
);

#[derive(Clone, Debug)]
pub struct InterfaceTypeDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub interface: Name<'a>,
    pub name: Recoverable<Name<'a>>,
    pub implements_interfaces: Option<ImplementsInterfaces<'a>>,
    pub directives: Option<Directives<'a>>,
    pub fields_definition: Option<FieldsDefinition<'a>>,
}

node!(
    InterfaceTypeDefinition,
    visit_interface_type_definition,
    description,
    interface,
    name,
    implements_interfaces,
    directives,
    fields_definition
);

#[derive(Clone, Debug)]
pub struct InterfaceTypeExtension<'a> {
    pub extend_interface: (Name<'a>, Name<'a>),
    pub name: Recoverable<Name<'a>>,
    pub implements_interfaces: Option<ImplementsInterfaces<'a>>,
    pub directives: Option<Directives<'a>>,
    pub fields_definition: Option<FieldsDefinition<'a>>,
}

node!(
    InterfaceTypeExtension,
    visit_interface_type_extension,
    extend_interface,
    name,
    implements_interfaces,
    directives,
    fields_definition
);

#[derive(Clone, Debug)]
pub struct UnionTypeDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub union_kw: Name<'a>,
    pub name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
    pub member_types: Option<UnionMemberTypes<'a>>,
}

node!(
    UnionTypeDefinition,
    visit_union_type_definition,
    description,
    union_kw,
    name,
    directives,
    member_types
);

#[derive(Clone, Debug)]
pub struct UnionMemberTypes<'a> {
    pub eq: Punctuator<'a>,
    pub pipe: Option<Punctuator<'a>>,
    pub first: Recoverable<NamedType<'a>>,
    pub types: Vec<(Punctuator<'a>, Recoverable<NamedType<'a>>)>,
}

node!(
    UnionMemberTypes,
    visit_union_member_types,
    eq,
    pipe,
    first,
    types
);

#[derive(Clone, Debug)]
pub struct UnionTypeExtension<'a> {
    pub extend_union: (Name<'a>, Name<'a>),
    pub name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
    pub member_types: Option<UnionMemberTypes<'a>>,
}

node!(
    UnionTypeExtension,
    visit_union_type_extension,
    extend_union,
    name,
    directives,
    member_types
);

#[derive(Clone, Debug)]
pub struct EnumTypeDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub enum_kw: Name<'a>,
    pub name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
    pub values_definition: Option<EnumValuesDefinition<'a>>,
}

node!(
    EnumTypeDefinition,
    visit_enum_type_definition,
    description,
    enum_kw,
    name,
    directives,
    values_definition
);

#[derive(Clone, Debug)]
pub struct EnumValuesDefinition<'a> {
    pub braces: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub definitions: Vec<EnumValueDefinition<'a>>,
}

node!(
    EnumValuesDefinition,
    visit_enum_values_definition,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct EnumValueDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub enum_value: EnumValue<'a>,
    pub directives: Option<Directives<'a>>,
}

node!(
    EnumValueDefinition,
    visit_enum_value_definition,
    description,
    enum_value,
    directives
);

#[derive(Clone, Debug)]
pub struct EnumTypeExtension<'a> {
    pub extend_enum: (Name<'a>, Name<'a>),
    pub name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
    pub values_definition: Option<EnumValuesDefinition<'a>>,
}

node!(
    EnumTypeExtension,
    visit_enum_type_extension,
    name,
    directives,
    values_definition
);

#[derive(Clone, Debug)]
pub struct InputObjectTypeDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub input: Name<'a>,
    pub name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
    pub fields_definition: Option<InputFieldsDefinition<'a>>,
}

node!(
    InputObjectTypeDefinition,
    visit_input_object_type_definition,
    description,
    input,
    name,
    directives,
    fields_definition
);

#[derive(Clone, Debug)]
pub struct InputFieldsDefinition<'a> {
    pub braces: (Punctuator<'a>, Recoverable<Punctuator<'a>>),
    pub definitions: Vec<InputValueDefinition<'a>>,
}

node!(
    InputFieldsDefinition,
    visit_input_fields_definition,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct InputObjectTypeExtension<'a> {
    pub extend_input: (Name<'a>, Name<'a>),
    pub name: Recoverable<Name<'a>>,
    pub directives: Option<Directives<'a>>,
    pub fields_definition: Option<InputFieldsDefinition<'a>>,
}

node!(
    InputObjectTypeExtension,
    visit_input_object_type_extension,
    extend_input,
    name,
    directives,
    fields_definition
);

#[derive(Clone, Debug)]
pub struct DirectiveDefinition<'a> {
    pub description: Option<Description<'a>>,
    pub directive: Name<'a>,
    pub at: Recoverable<Punctuator<'a>>,
    pub name: Recoverable<Name<'a>>,
    pub arguments_definition: Option<ArgumentsDefinition<'a>>,
    pub repeatable: Option<Name<'a>>,
    pub locations: Recoverable<DirectiveLocations<'a>>,
}

node!(
    DirectiveDefinition,
    visit_directive_definition,
    description,
    directive,
    at,
    name,
    arguments_definition,
    repeatable,
    locations
);

#[derive(Clone, Debug)]
pub struct DirectiveLocations<'a> {
    pub on: Name<'a>,
    pub pipe: Option<Punctuator<'a>>,
    pub first: Recoverable<DirectiveLocation<'a>>,
    pub locations: Vec<(Punctuator<'a>, Recoverable<DirectiveLocation<'a>>)>,
}

node!(
    DirectiveLocations,
    visit_directive_locations,
    on,
    pipe,
    first,
    locations
);

#[derive(Clone, Debug)]
pub enum DirectiveLocation<'a> {
    ExecutableDirectiveLocation(ExecutableDirectiveLocation<'a>),
    TypeSystemDirectiveLocation(TypeSystemDirectiveLocation<'a>),
}

node_enum!(
    DirectiveLocation,
    visit_directive_location,
    ExecutableDirectiveLocation,
    TypeSystemDirectiveLocation
);

#[derive(Clone, Debug)]
pub enum ExecutableDirectiveLocation<'a> {
    Query(Name<'a>),
    Mutation(Name<'a>),
    Subscription(Name<'a>),
    Field(Name<'a>),
    FragmentDefinition(Name<'a>),
    FragmentSpread(Name<'a>),
    InlineFragment(Name<'a>),
    VariableDefinition(Name<'a>),
}

node_enum!(
    ExecutableDirectiveLocation,
    visit_executable_directive_location,
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition
);

#[derive(Clone, Debug)]
pub enum TypeSystemDirectiveLocation<'a> {
    Schema(Name<'a>),
    Scalar(Name<'a>),
    Object(Name<'a>),
    FieldDefinition(Name<'a>),
    ArgumentDefinition(Name<'a>),
    Interface(Name<'a>),
    Union(Name<'a>),
    Enum(Name<'a>),
    EnumValue(Name<'a>),
    InputObject(Name<'a>),
    InputFieldDefinition(Name<'a>),
}

node_enum!(
    TypeSystemDirectiveLocation,
    visit_type_system_directive_location,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition
);
