use crate::lex::{Name, Punctuator, Token};

use super::{node, node_enum, node_unit, Node, Recoverable, Rest, Visit};

/// # 2.2
/// A GraphQL Document describes a complete file or request string operated on
/// by a GraphQL service or client. A document contains multiple definitions,
/// either executable or representative of a GraphQL type system.
#[derive(Clone, Debug, Default)]
pub struct Document<'a> {
    pub definitions: Vec<Recoverable<'a, Definition<'a>>>,
}

node!(Document, visit_document, definitions);

#[derive(Clone, Debug)]
pub enum Definition<'a> {
    ExecutableDefinition(ExecutableDefinition<'a>),
    // TypeSystemDefinitionOrExtension(TypeSystemDefinitionOrExtension<'a>),
}

node_enum!(Definition, visit_definition, ExecutableDefinition);

/// Documents are only executable by a GraphQL service if they are
/// `ExecutableDocument` and contain at least one `OperationDefinition`. A
/// Document which contains `TypeSystemDefinitionOrExtension` must not be
/// executed; GraphQL execution services which receive a Document containing
/// these should return a descriptive error.
#[derive(Clone, Debug)]
pub struct ExecutableDocument<'a> {
    pub definitions: Vec<Recoverable<'a, ExecutableDefinition<'a>>>,
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
    pub name: Option<Name<'a>>,
    pub variable_definitions: Option<VariableDefinitions<'a>>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: Recoverable<'a, SelectionSet<'a>>,
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
    pub braces: (Punctuator<'a>, Recoverable<'a, Punctuator<'a>>),
    pub selections: Vec<Recoverable<'a, Selection<'a>>>,
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
    pub name: Name<'a>,
    pub arguments: Option<Arguments<'a>>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: Option<SelectionSet<'a>>,

    pub rest: Rest<'a>,
}

node!(
    Field,
    visit_field,
    alias,
    name,
    arguments,
    directives,
    selection_set,
    rest
);

#[derive(Clone, Copy, Debug)]
pub struct Alias<'a> {
    pub name: Name<'a>,
    pub colon: Punctuator<'a>,
}

node!(Alias, visit_alias, name, colon);

#[derive(Clone, Debug)]
pub struct Arguments<'a> {
    pub parens: (Punctuator<'a>, Recoverable<'a, Punctuator<'a>>),
    pub items: Vec<Recoverable<'a, Argument<'a>>>,
}

node!(Arguments, visit_arguments, parens, items);

#[derive(Clone, Debug)]
pub struct Argument<'a> {
    pub name: Name<'a>,
    pub colon: Recoverable<'a, Punctuator<'a>>,
    pub value: Value<'a>,
}

node!(Argument, visit_argument, name, colon, value);

#[derive(Clone, Debug)]
pub struct FragmentSpread<'a> {
    pub dots: Punctuator<'a>,
    pub fragment_name: Name<'a>,
    pub directives: Option<Directives<'a>>,

    pub rest: Rest<'a>,
}

node!(
    FragmentSpread,
    visit_fragment_spread,
    dots,
    fragment_name,
    directives,
    rest
);

#[derive(Clone, Debug)]
pub struct InlineFragment<'a> {
    pub dots: Punctuator<'a>,
    pub type_condition: Option<TypeCondition<'a>>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: SelectionSet<'a>,

    pub rest: Rest<'a>,
}

node!(
    InlineFragment,
    visit_inline_fragment,
    dots,
    type_condition,
    directives,
    selection_set,
    rest
);

#[derive(Clone, Debug)]
pub struct FragmentDefinition<'a> {
    pub fragment: Name<'a>,
    pub fragment_name: Name<'a>,
    pub type_condition: TypeCondition<'a>,
    pub directives: Option<Directives<'a>>,
    pub selection_set: SelectionSet<'a>,
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
    pub named_type: Name<'a>,
}

node!(TypeCondition, visit_type_condition, on, named_type);

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Variable(Variable<'a>),
    // IntValue(IntValue<'a>),
    // FloatValue(FloatValue<'a>),
    // StringValue(StringValue<'a>),
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
    // IntValue,
    // FloatValue,
    // StringValue,
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
    pub brackets: (Punctuator<'a>, Recoverable<'a, Punctuator<'a>>),
    pub values: Vec<Value<'a>>,
}

node!(ListValue, visit_list_value, brackets, values);

#[derive(Clone, Debug)]
pub struct ObjectValue<'a> {
    pub braces: (Punctuator<'a>, Recoverable<'a, Punctuator<'a>>),
    pub object_fields: Vec<ObjectField<'a>>,
}

node!(ObjectValue, visit_object_value, braces, object_fields);

#[derive(Clone, Debug)]
pub struct ObjectField<'a> {
    pub name: Name<'a>,
    pub colon: Punctuator<'a>,
    pub value: Value<'a>,
}

node!(ObjectField, visit_object_field, name, colon, value);

#[derive(Clone, Debug)]
pub struct VariableDefinitions<'a> {
    pub parens: (Punctuator<'a>, Recoverable<'a, Punctuator<'a>>),
    pub variable_definitions: Vec<Recoverable<'a, VariableDefinition<'a>>>,
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
    pub colon: Punctuator<'a>,
    pub ty: Type<'a>,
    pub default_value: Option<DefaultValue<'a>>,
    pub directives: Option<Directives<'a>>,

    pub rest: Result<(), Vec<Token<'a>>>,
}

node!(
    VariableDefinition,
    visit_variable_definition,
    variable,
    colon,
    ty,
    default_value,
    directives,
    rest
);

#[derive(Clone, Debug)]
pub struct Variable<'a> {
    pub dollar: Recoverable<'a, Punctuator<'a>>,
    pub name: Name<'a>,

    pub rest: Result<(), Vec<Token<'a>>>,
}

node!(Variable, visit_variable, dollar, name, rest);

#[derive(Clone, Debug)]
pub struct DefaultValue<'a> {
    pub eq: Punctuator<'a>,
    pub value: Value<'a>,

    pub rest: Result<(), Vec<Token<'a>>>,
}

node!(DefaultValue, visit_default_value, eq, value, rest);

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
    pub brackets: (Punctuator<'a>, Recoverable<'a, Punctuator<'a>>),
    pub ty: Type<'a>,
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
    pub name: Name<'a>,
    pub arguments: Option<Arguments<'a>>,

    pub rest: Result<(), Vec<Token<'a>>>,
}

node!(Directive, visit_directive, at, name, arguments, rest);
