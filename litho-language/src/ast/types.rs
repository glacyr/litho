use std::fmt::{Display, Formatter, Result};
use std::sync::Arc;

pub use crate::lex::{FloatValue, IntValue, Name, Punctuator, Span, StringValue};

use super::{node, node_enum, node_unit, Node, Visit};

#[derive(Debug, Clone)]
pub enum Missing {
    Unknown,
    Simple(&'static str, &'static str),
    Delimiter(&'static str, &'static str, Span, &'static str),
}

impl Missing {
    pub fn delimiter_complaint<N, T>(
        message: &'static str,
        first_label: &'static str,
        second_label: &'static str,
    ) -> impl Fn(&N) -> Missing
    where
        N: Node<T>,
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
pub struct Document<T> {
    pub definitions: Vec<Definition<T>>,
}

node!(Document, visit_document, definitions);

#[derive(Clone, Debug)]
pub enum Definition<T> {
    ExecutableDefinition(ExecutableDefinition<T>),
    TypeSystemDefinitionOrExtension(TypeSystemDefinitionOrExtension<T>),
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
pub struct ExecutableDocument<T> {
    pub definitions: Vec<ExecutableDefinition<T>>,
}

node!(ExecutableDocument, visit_executable_document, definitions);

#[derive(Clone, Debug)]
pub enum ExecutableDefinition<T> {
    OperationDefinition(Arc<OperationDefinition<T>>),
    FragmentDefinition(FragmentDefinition<T>),
}

node_enum!(
    ExecutableDefinition,
    visit_executable_definition,
    OperationDefinition,
    FragmentDefinition
);

#[derive(Clone, Debug)]
pub struct OperationDefinition<T> {
    pub ty: Option<OperationType<T>>,
    pub name: Recoverable<Name<T>>,
    pub variable_definitions: Option<VariableDefinitions<T>>,
    pub directives: Option<Directives<T>>,
    pub selection_set: Recoverable<Arc<SelectionSet<T>>>,
}

node!(
    Arc<OperationDefinition>,
    visit_operation_definition + post_visit_operation_definition,
    ty,
    name,
    variable_definitions,
    directives,
    selection_set
);

#[derive(Clone, Copy, Debug)]
pub enum OperationType<T> {
    Query(Name<T>),
    Mutation(Name<T>),
    Subscription(Name<T>),
}

node_enum!(
    OperationType,
    visit_operation_type,
    Query,
    Mutation,
    Subscription
);

#[derive(Clone, Debug)]
pub struct SelectionSet<T> {
    pub braces: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub selections: Vec<Selection<T>>,
}

node!(Arc<SelectionSet>, visit_selection_set, braces, selections);

#[derive(Clone, Debug)]
pub enum Selection<T> {
    Field(Arc<Field<T>>),
    FragmentSpread(FragmentSpread<T>),
    InlineFragment(InlineFragment<T>),
}

node_enum!(
    Selection,
    visit_selection,
    Field,
    FragmentSpread,
    InlineFragment
);

#[derive(Clone, Debug)]
pub struct Field<T> {
    pub alias: Option<Alias<T>>,
    pub name: Recoverable<Name<T>>,
    pub arguments: Option<Arc<Arguments<T>>>,
    pub directives: Option<Directives<T>>,
    pub selection_set: Option<Arc<SelectionSet<T>>>,
}

node!(
    Arc<Field>,
    visit_field + post_visit_field,
    alias,
    name,
    arguments,
    directives,
    selection_set
);

#[derive(Clone, Debug)]
pub struct Alias<T> {
    pub name: Name<T>,
    pub colon: Punctuator<T>,
}

node!(Alias, visit_alias, name, colon);

#[derive(Clone, Debug)]
pub struct Arguments<T> {
    pub parens: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub items: Vec<Argument<T>>,
}

node!(Arc<Arguments>, visit_arguments, parens, items);

#[derive(Clone, Debug)]
pub struct Argument<T> {
    pub name: Name<T>,
    pub colon: Recoverable<Punctuator<T>>,
    pub value: Recoverable<Value<T>>,
}

node!(Argument, visit_argument, name, colon, value);

#[derive(Clone, Debug)]
pub struct FragmentSpread<T> {
    pub dots: Punctuator<T>,
    pub fragment_name: Name<T>,
    pub directives: Option<Directives<T>>,
}

node!(
    FragmentSpread,
    visit_fragment_spread,
    dots,
    fragment_name,
    directives
);

#[derive(Clone, Debug)]
pub struct InlineFragment<T> {
    pub dots: Punctuator<T>,
    pub type_condition: Option<TypeCondition<T>>,
    pub directives: Option<Directives<T>>,
    pub selection_set: Recoverable<Arc<SelectionSet<T>>>,
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
pub struct FragmentDefinition<T> {
    pub fragment: Name<T>,
    pub fragment_name: Recoverable<Name<T>>,
    pub type_condition: Recoverable<TypeCondition<T>>,
    pub directives: Option<Directives<T>>,
    pub selection_set: Recoverable<Arc<SelectionSet<T>>>,
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
pub struct TypeCondition<T> {
    pub on: Name<T>,
    pub named_type: Recoverable<Name<T>>,
}

node!(TypeCondition, visit_type_condition, on, named_type);

#[derive(Clone, Debug)]
pub enum Value<T> {
    Variable(Variable<T>),
    IntValue(IntValue<T>),
    FloatValue(FloatValue<T>),
    StringValue(StringValue<T>),
    BooleanValue(BooleanValue<T>),
    NullValue(NullValue<T>),
    EnumValue(EnumValue<T>),
    ListValue(ListValue<T>),
    ObjectValue(ObjectValue<T>),
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
pub enum BooleanValue<T> {
    True(Name<T>),
    False(Name<T>),
}

node_enum!(BooleanValue, visit_boolean_value, True, False);

#[derive(Clone, Debug)]
pub struct NullValue<T>(pub Name<T>);

node_unit!(NullValue, visit_null_value);

#[derive(Clone, Debug)]
pub struct EnumValue<T>(pub Name<T>);

node_unit!(EnumValue, visit_enum_value);

#[derive(Clone, Debug)]
pub struct ListValue<T> {
    pub brackets: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub values: Vec<Value<T>>,
}

node!(ListValue, visit_list_value, brackets, values);

#[derive(Clone, Debug)]
pub struct ObjectValue<T> {
    pub braces: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub object_fields: Vec<ObjectField<T>>,
}

node!(ObjectValue, visit_object_value, braces, object_fields);

#[derive(Clone, Debug)]
pub struct ObjectField<T> {
    pub name: Name<T>,
    pub colon: Recoverable<Punctuator<T>>,
    pub value: Recoverable<Value<T>>,
}

node!(ObjectField, visit_object_field, name, colon, value);

#[derive(Clone, Debug)]
pub struct VariableDefinitions<T> {
    pub parens: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub variable_definitions: Vec<VariableDefinition<T>>,
}

node!(
    VariableDefinitions,
    visit_variable_definitions,
    parens,
    variable_definitions
);

#[derive(Clone, Debug)]
pub struct VariableDefinition<T> {
    pub variable: Variable<T>,
    pub colon: Recoverable<Punctuator<T>>,
    pub ty: Recoverable<Type<T>>,
    pub default_value: Option<DefaultValue<T>>,
    pub directives: Option<Directives<T>>,
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
pub struct Variable<T> {
    pub dollar: Punctuator<T>,
    pub name: Name<T>,
}

node!(Variable, visit_variable, dollar, name);

#[derive(Clone, Debug)]
pub struct DefaultValue<T> {
    pub eq: Punctuator<T>,
    pub value: Recoverable<Value<T>>,
}

node!(DefaultValue, visit_default_value, eq, value);

#[derive(Clone, Debug)]
pub enum Type<T> {
    Named(NamedType<T>),
    List(Box<ListType<T>>),
    NonNull(Box<NonNullType<T>>),
}

impl<T> Type<T> {
    pub fn name(&self) -> Option<&T> {
        self.named_type().map(|ty| ty.0.as_ref())
    }

    pub fn named_type(&self) -> Option<&NamedType<T>> {
        match self {
            Type::Named(ty) => Some(ty),
            Type::List(ty) => ty.ty.ok().and_then(Type::named_type),
            Type::NonNull(ty) => ty.ty.named_type(),
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(self, Type::NonNull(_))
    }
}

impl<T> Type<T>
where
    T: Eq,
{
    pub fn is_invariant(&self, other: &Type<T>) -> bool {
        match (self, other) {
            (Type::Named(lhs), Type::Named(rhs)) => lhs.0.as_ref() == rhs.0.as_ref(),
            (Type::List(lhs), Type::List(rhs)) => lhs
                .ty
                .ok()
                .zip(rhs.ty.ok())
                .map(|(lhs, rhs)| lhs.is_invariant(rhs))
                .unwrap_or_default(),
            (Type::NonNull(lhs), Type::NonNull(rhs)) => lhs.ty.is_invariant(&rhs.ty),
            (_, _) => false,
        }
    }
}

impl<T> Display for Type<T>
where
    T: ToString,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Type::Named(ty) => f.write_str(&ty.0.as_ref().to_string()),
            Type::List(ty) => f.write_fmt(format_args!(
                "[{}]",
                ty.ty
                    .ok()
                    .map(|ty| ty.to_string())
                    .unwrap_or("...".to_owned())
            )),
            Type::NonNull(ty) => f.write_fmt(format_args!("{}!", ty.ty.to_string())),
        }
    }
}

node_enum!(Type, visit_type, Named, List, NonNull);

#[derive(Clone, Debug)]
pub struct NamedType<T>(pub Name<T>);

node_unit!(NamedType, visit_named_type);

#[derive(Clone, Debug)]
pub struct ListType<T> {
    pub brackets: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub ty: Recoverable<Type<T>>,
}

node!(ListType, visit_list_type, brackets, ty);

#[derive(Clone, Debug)]
pub struct NonNullType<T> {
    pub ty: Type<T>,
    pub bang: Punctuator<T>,
}

node!(NonNullType, visit_non_null_type, ty, bang);

#[derive(Clone, Debug)]
pub struct Directives<T> {
    pub directives: Vec<Directive<T>>,
}

node!(Directives, visit_directives, directives);

#[derive(Clone, Debug)]
pub struct Directive<T> {
    pub at: Punctuator<T>,
    pub name: Recoverable<Name<T>>,
    pub arguments: Option<Arc<Arguments<T>>>,
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
pub struct TypeSystemDocument<T> {
    pub definitions: Vec<TypeSystemDefinition<T>>,
}

node!(TypeSystemDocument, visit_type_system_document, definitions);

#[derive(Clone, Debug)]
pub enum TypeSystemDefinition<T> {
    SchemaDefinition(SchemaDefinition<T>),
    TypeDefinition(Arc<TypeDefinition<T>>),
    DirectiveDefinition(DirectiveDefinition<T>),
}

node_enum!(
    TypeSystemDefinition,
    visit_type_system_definition,
    SchemaDefinition,
    TypeDefinition,
    DirectiveDefinition
);

#[derive(Clone, Debug)]
pub struct TypeSystemExtensionDocument<T> {
    pub definitions: Vec<TypeSystemDefinitionOrExtension<T>>,
}

node!(
    TypeSystemExtensionDocument,
    visit_type_system_extension_document,
    definitions
);

#[derive(Clone, Debug)]
pub enum TypeSystemDefinitionOrExtension<T> {
    TypeSystemDefinition(TypeSystemDefinition<T>),
    TypeSystemExtension(TypeSystemExtension<T>),
}

node_enum!(
    TypeSystemDefinitionOrExtension,
    visit_type_system_definition_or_extension,
    TypeSystemDefinition,
    TypeSystemExtension
);

#[derive(Clone, Debug)]
pub enum TypeSystemExtension<T> {
    SchemaExtension(SchemaExtension<T>),
    TypeExtension(TypeExtension<T>),
}

node_enum!(
    TypeSystemExtension,
    visit_type_system_extension,
    SchemaExtension,
    TypeExtension
);

#[derive(Clone, Debug)]
pub struct Description<T>(pub StringValue<T>);

impl<T> ToString for Description<T>
where
    T: ToString,
{
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

node_unit!(Description, visit_description);

#[derive(Clone, Debug)]
pub struct SchemaDefinition<T> {
    pub description: Option<Description<T>>,
    pub schema: Name<T>,
    pub directives: Option<Directives<T>>,
    pub type_definitions: Recoverable<RootOperationTypeDefinitions<T>>,
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
pub struct RootOperationTypeDefinitions<T> {
    pub braces: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub definitions: Vec<RootOperationTypeDefinition<T>>,
}

node!(
    RootOperationTypeDefinitions,
    visit_root_operation_type_definitions,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct RootOperationTypeDefinition<T> {
    pub operation_type: OperationType<T>,
    pub colon: Recoverable<Punctuator<T>>,
    pub named_type: Recoverable<NamedType<T>>,
}

node!(
    RootOperationTypeDefinition,
    visit_root_operation_type_definition,
    operation_type,
    colon,
    named_type
);

#[derive(Clone, Debug)]
pub struct SchemaExtension<T> {
    pub extend_schema: (Name<T>, Name<T>),
    pub directives: Option<Directives<T>>,
    pub type_definitions: Option<RootOperationTypeDefinitions<T>>,
}

node!(
    SchemaExtension,
    visit_schema_extension,
    extend_schema,
    directives,
    type_definitions
);

#[derive(Clone, Debug)]
pub enum TypeDefinition<T> {
    ScalarTypeDefinition(ScalarTypeDefinition<T>),
    ObjectTypeDefinition(ObjectTypeDefinition<T>),
    InterfaceTypeDefinition(InterfaceTypeDefinition<T>),
    UnionTypeDefinition(UnionTypeDefinition<T>),
    EnumTypeDefinition(EnumTypeDefinition<T>),
    InputObjectTypeDefinition(InputObjectTypeDefinition<T>),
}

node_enum!(
    Arc<TypeDefinition>,
    visit_type_definition,
    ScalarTypeDefinition,
    ObjectTypeDefinition,
    InterfaceTypeDefinition,
    UnionTypeDefinition,
    EnumTypeDefinition,
    InputObjectTypeDefinition
);

impl<T> TypeDefinition<T> {
    pub fn description(&self) -> Option<&Description<T>> {
        match self {
            TypeDefinition::ScalarTypeDefinition(definition) => definition.description.as_ref(),
            TypeDefinition::ObjectTypeDefinition(definition) => definition.description.as_ref(),
            TypeDefinition::InterfaceTypeDefinition(definition) => definition.description.as_ref(),
            TypeDefinition::UnionTypeDefinition(definition) => definition.description.as_ref(),
            TypeDefinition::EnumTypeDefinition(definition) => definition.description.as_ref(),
            TypeDefinition::InputObjectTypeDefinition(definition) => {
                definition.description.as_ref()
            }
        }
    }

    pub fn name(&self) -> &Recoverable<Name<T>> {
        match self {
            TypeDefinition::ScalarTypeDefinition(definition) => &definition.name,
            TypeDefinition::ObjectTypeDefinition(definition) => &definition.name,
            TypeDefinition::InterfaceTypeDefinition(definition) => &definition.name,
            TypeDefinition::UnionTypeDefinition(definition) => &definition.name,
            TypeDefinition::EnumTypeDefinition(definition) => &definition.name,
            TypeDefinition::InputObjectTypeDefinition(definition) => &definition.name,
        }
    }

    pub fn is_input(&self) -> bool {
        match self {
            TypeDefinition::EnumTypeDefinition(_)
            | TypeDefinition::InputObjectTypeDefinition(_)
            | TypeDefinition::ScalarTypeDefinition(_) => true,
            TypeDefinition::InterfaceTypeDefinition(_)
            | TypeDefinition::ObjectTypeDefinition(_)
            | TypeDefinition::UnionTypeDefinition(_) => false,
        }
    }

    pub fn is_output(&self) -> bool {
        match self {
            TypeDefinition::InputObjectTypeDefinition(_) => false,
            TypeDefinition::EnumTypeDefinition(_)
            | TypeDefinition::InterfaceTypeDefinition(_)
            | TypeDefinition::ObjectTypeDefinition(_)
            | TypeDefinition::ScalarTypeDefinition(_)
            | TypeDefinition::UnionTypeDefinition(_) => true,
        }
    }

    pub fn implements_interfaces(&self) -> Option<&ImplementsInterfaces<T>> {
        match self {
            TypeDefinition::InterfaceTypeDefinition(definition) => {
                definition.implements_interfaces.as_ref()
            }
            TypeDefinition::ObjectTypeDefinition(definition) => {
                definition.implements_interfaces.as_ref()
            }
            _ => None,
        }
    }

    pub fn fields_definition(&self) -> Option<&FieldsDefinition<T>> {
        match self {
            TypeDefinition::InterfaceTypeDefinition(definition) => {
                definition.fields_definition.as_ref()
            }
            TypeDefinition::ObjectTypeDefinition(definition) => {
                definition.fields_definition.as_ref()
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeExtension<T> {
    ScalarTypeExtension(ScalarTypeExtension<T>),
    ObjectTypeExtension(ObjectTypeExtension<T>),
    InterfaceTypeExtension(InterfaceTypeExtension<T>),
    UnionTypeExtension(UnionTypeExtension<T>),
    EnumTypeExtension(EnumTypeExtension<T>),
    InputObjectTypeExtension(InputObjectTypeExtension<T>),
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

impl<T> TypeExtension<T> {
    pub fn name(&self) -> &Recoverable<Name<T>> {
        match self {
            TypeExtension::ScalarTypeExtension(extension) => &extension.name,
            TypeExtension::ObjectTypeExtension(extension) => &extension.name,
            TypeExtension::InterfaceTypeExtension(extension) => &extension.name,
            TypeExtension::UnionTypeExtension(extension) => &extension.name,
            TypeExtension::EnumTypeExtension(extension) => &extension.name,
            TypeExtension::InputObjectTypeExtension(extension) => &extension.name,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition<T> {
    pub description: Option<Description<T>>,
    pub scalar: Name<T>,
    pub name: Recoverable<Name<T>>,
    pub directives: Option<Directives<T>>,
}

node!(
    ScalarTypeDefinition,
    visit_scalar_type_definition,
    scalar,
    name,
    directives
);

#[derive(Clone, Debug)]
pub struct ScalarTypeExtension<T> {
    pub extend_scalar: (Name<T>, Name<T>),
    pub name: Recoverable<Name<T>>,
    pub directives: Recoverable<Directives<T>>,
}

node!(
    ScalarTypeExtension,
    visit_scalar_type_extension,
    extend_scalar,
    name,
    directives
);

#[derive(Clone, Debug)]
pub struct ObjectTypeDefinition<T> {
    pub description: Option<Description<T>>,
    pub ty: Name<T>,
    pub name: Recoverable<Name<T>>,
    pub implements_interfaces: Option<ImplementsInterfaces<T>>,
    pub directives: Option<Directives<T>>,
    pub fields_definition: Option<FieldsDefinition<T>>,
}

impl<T> ObjectTypeDefinition<T>
where
    T: Eq,
{
    pub fn implements_interface(&self, name: &T) -> bool {
        self.implements_interfaces
            .as_ref()
            .map(|interfaces| interfaces.implements_interface(name))
            .unwrap_or_default()
    }
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
pub struct ImplementsInterfaces<T> {
    pub implements: Name<T>,
    pub ampersand: Option<Punctuator<T>>,
    pub first: Recoverable<NamedType<T>>,
    pub types: Vec<(Punctuator<T>, Recoverable<NamedType<T>>)>,
}

impl<T> ImplementsInterfaces<T> {
    pub fn named_types(&self) -> impl Iterator<Item = &NamedType<T>> {
        self.first
            .ok()
            .into_iter()
            .chain(self.types.iter().flat_map(|(_, ty)| ty.ok()))
    }

    pub fn types(&self) -> impl Iterator<Item = &T> {
        self.named_types().map(|ty| ty.0.as_ref())
    }
}

impl<T> ImplementsInterfaces<T>
where
    T: Eq,
{
    pub fn implements_interface(&self, name: &T) -> bool {
        self.types().any(|ty| ty == name)
    }
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
pub struct FieldsDefinition<T> {
    pub braces: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub definitions: Vec<Arc<FieldDefinition<T>>>,
}

impl<T> FieldsDefinition<T>
where
    T: Eq,
{
    pub fn field(&self, name: &T) -> Option<&FieldDefinition<T>> {
        self.definitions
            .iter()
            .find(|field| field.name.as_ref() == name)
            .map(AsRef::as_ref)
    }
}

node!(
    FieldsDefinition,
    visit_fields_definition,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct FieldDefinition<T> {
    pub description: Option<Description<T>>,
    pub name: Name<T>,
    pub arguments_definition: Option<Arc<ArgumentsDefinition<T>>>,
    pub colon: Recoverable<Punctuator<T>>,
    pub ty: Recoverable<Type<T>>,
    pub directives: Option<Directives<T>>,
}

node!(
    Arc<FieldDefinition>,
    visit_field_definition,
    description,
    name,
    arguments_definition,
    colon,
    ty,
    directives
);

#[derive(Clone, Debug)]
pub struct ArgumentsDefinition<T> {
    pub parens: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub definitions: Vec<InputValueDefinition<T>>,
}

impl<T> ArgumentsDefinition<T>
where
    T: Eq,
{
    pub fn argument(&self, name: &T) -> Option<&InputValueDefinition<T>> {
        self.definitions
            .iter()
            .find(|def| def.name.as_ref() == name)
    }
}

node!(
    Arc<ArgumentsDefinition>,
    visit_arguments_definition,
    parens,
    definitions
);

#[derive(Clone, Debug)]
pub struct InputValueDefinition<T> {
    pub description: Option<Description<T>>,
    pub name: Name<T>,
    pub colon: Recoverable<Punctuator<T>>,
    pub ty: Recoverable<Type<T>>,
    pub default_value: Option<DefaultValue<T>>,
    pub directives: Option<Directives<T>>,
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
pub struct ObjectTypeExtension<T> {
    pub extend_type: (Name<T>, Name<T>),
    pub name: Recoverable<Name<T>>,
    pub implements_interfaces: Option<ImplementsInterfaces<T>>,
    pub directives: Option<Directives<T>>,
    pub fields_definition: Option<FieldsDefinition<T>>,
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
pub struct InterfaceTypeDefinition<T> {
    pub description: Option<Description<T>>,
    pub interface: Name<T>,
    pub name: Recoverable<Name<T>>,
    pub implements_interfaces: Option<ImplementsInterfaces<T>>,
    pub directives: Option<Directives<T>>,
    pub fields_definition: Option<FieldsDefinition<T>>,
}

impl<T> InterfaceTypeDefinition<T>
where
    T: Eq,
{
    pub fn implements_interface(&self, name: &T) -> bool {
        self.implements_interfaces
            .as_ref()
            .map(|interfaces| interfaces.implements_interface(name))
            .unwrap_or_default()
    }
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
pub struct InterfaceTypeExtension<T> {
    pub extend_interface: (Name<T>, Name<T>),
    pub name: Recoverable<Name<T>>,
    pub implements_interfaces: Option<ImplementsInterfaces<T>>,
    pub directives: Option<Directives<T>>,
    pub fields_definition: Option<FieldsDefinition<T>>,
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
pub struct UnionTypeDefinition<T> {
    pub description: Option<Description<T>>,
    pub union_kw: Name<T>,
    pub name: Recoverable<Name<T>>,
    pub directives: Option<Directives<T>>,
    pub member_types: Option<UnionMemberTypes<T>>,
}

impl<T> UnionTypeDefinition<T>
where
    T: Eq,
{
    pub fn includes_member_type(&self, name: &T) -> bool {
        self.member_types
            .as_ref()
            .map(|types| types.includes_member_type(name))
            .unwrap_or_default()
    }
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
pub struct UnionMemberTypes<T> {
    pub eq: Punctuator<T>,
    pub pipe: Option<Punctuator<T>>,
    pub first: Recoverable<NamedType<T>>,
    pub types: Vec<(Punctuator<T>, Recoverable<NamedType<T>>)>,
}

impl<T> UnionMemberTypes<T> {
    pub fn named_types(&self) -> impl Iterator<Item = &NamedType<T>> {
        self.first
            .ok()
            .into_iter()
            .chain(self.types.iter().flat_map(|(_, ty)| ty.ok()))
    }

    pub fn types(&self) -> impl Iterator<Item = &T> {
        self.named_types().map(|ty| ty.0.as_ref())
    }
}

impl<T> UnionMemberTypes<T>
where
    T: Eq,
{
    pub fn includes_member_type(&self, name: &T) -> bool {
        self.types().any(|ty| ty == name)
    }
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
pub struct UnionTypeExtension<T> {
    pub extend_union: (Name<T>, Name<T>),
    pub name: Recoverable<Name<T>>,
    pub directives: Option<Directives<T>>,
    pub member_types: Option<UnionMemberTypes<T>>,
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
pub struct EnumTypeDefinition<T> {
    pub description: Option<Description<T>>,
    pub enum_kw: Name<T>,
    pub name: Recoverable<Name<T>>,
    pub directives: Option<Directives<T>>,
    pub values_definition: Option<EnumValuesDefinition<T>>,
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
pub struct EnumValuesDefinition<T> {
    pub braces: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub definitions: Vec<EnumValueDefinition<T>>,
}

node!(
    EnumValuesDefinition,
    visit_enum_values_definition,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct EnumValueDefinition<T> {
    pub description: Option<Description<T>>,
    pub enum_value: EnumValue<T>,
    pub directives: Option<Directives<T>>,
}

node!(
    EnumValueDefinition,
    visit_enum_value_definition,
    description,
    enum_value,
    directives
);

#[derive(Clone, Debug)]
pub struct EnumTypeExtension<T> {
    pub extend_enum: (Name<T>, Name<T>),
    pub name: Recoverable<Name<T>>,
    pub directives: Option<Directives<T>>,
    pub values_definition: Option<EnumValuesDefinition<T>>,
}

node!(
    EnumTypeExtension,
    visit_enum_type_extension,
    name,
    directives,
    values_definition
);

#[derive(Clone, Debug)]
pub struct InputObjectTypeDefinition<T> {
    pub description: Option<Description<T>>,
    pub input: Name<T>,
    pub name: Recoverable<Name<T>>,
    pub directives: Option<Directives<T>>,
    pub fields_definition: Option<InputFieldsDefinition<T>>,
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
pub struct InputFieldsDefinition<T> {
    pub braces: (Punctuator<T>, Recoverable<Punctuator<T>>),
    pub definitions: Vec<InputValueDefinition<T>>,
}

node!(
    InputFieldsDefinition,
    visit_input_fields_definition,
    braces,
    definitions
);

#[derive(Clone, Debug)]
pub struct InputObjectTypeExtension<T> {
    pub extend_input: (Name<T>, Name<T>),
    pub name: Recoverable<Name<T>>,
    pub directives: Option<Directives<T>>,
    pub fields_definition: Option<InputFieldsDefinition<T>>,
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
pub struct DirectiveDefinition<T> {
    pub description: Option<Description<T>>,
    pub directive: Name<T>,
    pub at: Recoverable<Punctuator<T>>,
    pub name: Recoverable<Name<T>>,
    pub arguments_definition: Option<Arc<ArgumentsDefinition<T>>>,
    pub repeatable: Option<Name<T>>,
    pub locations: Recoverable<DirectiveLocations<T>>,
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
pub struct DirectiveLocations<T> {
    pub on: Name<T>,
    pub pipe: Option<Punctuator<T>>,
    pub first: Recoverable<DirectiveLocation<T>>,
    pub locations: Vec<(Punctuator<T>, Recoverable<DirectiveLocation<T>>)>,
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
pub enum DirectiveLocation<T> {
    ExecutableDirectiveLocation(ExecutableDirectiveLocation<T>),
    TypeSystemDirectiveLocation(TypeSystemDirectiveLocation<T>),
}

node_enum!(
    DirectiveLocation,
    visit_directive_location,
    ExecutableDirectiveLocation,
    TypeSystemDirectiveLocation
);

#[derive(Clone, Debug)]
pub enum ExecutableDirectiveLocation<T> {
    Query(Name<T>),
    Mutation(Name<T>),
    Subscription(Name<T>),
    Field(Name<T>),
    FragmentDefinition(Name<T>),
    FragmentSpread(Name<T>),
    InlineFragment(Name<T>),
    VariableDefinition(Name<T>),
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
pub enum TypeSystemDirectiveLocation<T> {
    Schema(Name<T>),
    Scalar(Name<T>),
    Object(Name<T>),
    FieldDefinition(Name<T>),
    ArgumentDefinition(Name<T>),
    Interface(Name<T>),
    Union(Name<T>),
    Enum(Name<T>),
    EnumValue(Name<T>),
    InputObject(Name<T>),
    InputFieldDefinition(Name<T>),
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
