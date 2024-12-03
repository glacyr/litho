use std::fmt::{Display, Formatter, Result};

use arbitrary::Arbitrary;
use litho_diagnostics::Diagnostic;

pub use crate::lex::{FloatValue, IntValue, Name, Punctuator, Span, StringValue};

use super::context::AsPtr;
use super::mock::{arbitrary_at_least_one, arbitrary_present_at_least_one, arbitrary_punctuators};
use super::{node, node_arc, node_enum, node_unit, ContextValue, List, Node, Shared, Visit};

#[derive(Clone, Copy, Debug)]
pub enum Missing {
    Unknown,
    Unary(fn(Span) -> Diagnostic<Span>),
    Binary(fn(Span, Span) -> Diagnostic<Span>, Span),
}

impl Missing {
    #[inline]
    pub fn unary(factory: fn(Span) -> Diagnostic<Span>) -> Missing {
        Missing::Unary(factory)
    }

    #[inline(always)]
    pub fn binary<'a, N, T>(factory: fn(Span, Span) -> Diagnostic<Span>) -> impl Fn(&N) -> Missing
    where
        N: Node<'a, T>,
        T: ContextValue<'a>,
    {
        move |left| Missing::Binary(factory, left.span())
    }
}

impl Default for Missing {
    fn default() -> Self {
        Missing::Unknown
    }
}

pub trait Spanned {
    fn span(&mut self) -> Span;
}

#[derive(Debug, Clone)]
pub struct MissingToken {
    pub span: Span,
    pub missing: Missing,
}

impl MissingToken {
    pub fn to_diagnostic(&self) -> Diagnostic<Span> {
        match self.missing {
            Missing::Unknown => unreachable!(),
            Missing::Unary(factory) => factory(self.span),
            Missing::Binary(factory, span) => factory(span, self.span),
        }
    }
}

pub type Recoverable<T> = wrom::Recoverable<T, MissingToken>;

/// # 2.2
/// A GraphQL Document describes a complete file or request string operated on
/// by a GraphQL service or client. A document contains multiple definitions,
/// either executable or representative of a GraphQL type system.
#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Document<'a, T>
where
    T: ContextValue<'a>,
{
    pub definitions: List<'a, T, Shared<'a, T, Definition<'a, T>>>,
}

node!(Document, visit_document, definitions);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum Definition<'a, T>
where
    T: ContextValue<'a>,
{
    ExecutableDefinition(ExecutableDefinition<'a, T>),
    TypeSystemDefinitionOrExtension(TypeSystemDefinitionOrExtension<'a, T>),
}

node_enum!(
    Shared<'a, T, Definition>,
    visit_definition,
    ExecutableDefinition,
    TypeSystemDefinitionOrExtension
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DefinitionId(usize);

impl<'a, T> Definition<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn id(this: &Shared<'a, T, Self>) -> DefinitionId {
        DefinitionId(this.as_ptr())
    }
}

/// Documents are only executable by a GraphQL service if they are
/// `ExecutableDocument` and contain at least one `OperationDefinition`. A
/// Document which contains `TypeSystemDefinitionOrExtension` must not be
/// executed; GraphQL execution services which receive a Document containing
/// these should return a descriptive error.
#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ExecutableDocument<'a, T>
where
    T: ContextValue<'a>,
{
    pub definitions: List<'a, T, ExecutableDefinition<'a, T>>,
}

node!(ExecutableDocument, visit_executable_document, definitions);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum ExecutableDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    OperationDefinition(Shared<'a, T, OperationDefinition<'a, T>>),
    FragmentDefinition(Shared<'a, T, FragmentDefinition<'a, T>>),
}

node_enum!(
    ExecutableDefinition,
    visit_executable_definition,
    OperationDefinition,
    FragmentDefinition
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct OperationDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Some(u.arbitrary()?))]
    pub ty: Option<OperationType<'a, T>>,
    pub name: Option<Name<'a, T>>,
    pub variable_definitions: Option<VariableDefinitions<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
    pub selection_set: Recoverable<Shared<'a, T, SelectionSet<'a, T>>>,
}

node!(
    Shared<'a, T, OperationDefinition>,
    visit_operation_definition + post_visit_operation_definition,
    ty,
    name,
    variable_definitions,
    directives,
    selection_set
);

#[derive(Clone, Copy, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum OperationType<'a, T>
where
    T: ContextValue<'a>,
{
    Query(#[arbitrary(value = Name::new("query"))] Name<'a, T>),
    Mutation(#[arbitrary(value = Name::new("mutation"))] Name<'a, T>),
    Subscription(#[arbitrary(value = Name::new("subscription"))] Name<'a, T>),
}

node_enum!(
    OperationType,
    visit_operation_type,
    Query,
    Mutation,
    Subscription
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct SelectionSet<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("{", "}"))]
    pub braces: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),
    pub selections: List<'a, T, Selection<'a, T>>,
}

node!(
    Shared<'a, T, SelectionSet>,
    visit_selection_set,
    braces,
    selections
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum Selection<'a, T>
where
    T: ContextValue<'a>,
{
    Field(Shared<'a, T, Field<'a, T>>),
    FragmentSpread(Shared<'a, T, FragmentSpread<'a, T>>),
    InlineFragment(InlineFragment<'a, T>),
}

node_enum!(
    Selection,
    visit_selection,
    Field,
    FragmentSpread,
    InlineFragment
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Field<'a, T>
where
    T: ContextValue<'a>,
{
    pub alias: Option<Alias<'a, T>>,
    pub name: Recoverable<Name<'a, T>>,
    pub arguments: Option<Shared<'a, T, Arguments<'a, T>>>,
    pub directives: Option<Directives<'a, T>>,
    pub selection_set: Option<Shared<'a, T, SelectionSet<'a, T>>>,
}

node!(
    Shared<'a, T, Field>,
    visit_field + post_visit_field,
    alias,
    name,
    arguments,
    directives,
    selection_set
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Alias<'a, T>
where
    T: ContextValue<'a>,
{
    pub name: Name<'a, T>,

    #[arbitrary(value = Punctuator::new(":"))]
    pub colon: Punctuator<'a, T>,
}

node!(Alias, visit_alias, name, colon);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Arguments<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("(", ")"))]
    pub parens: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),
    pub items: List<'a, T, Shared<'a, T, Argument<'a, T>>>,
}

node!(Shared<'a, T, Arguments>, visit_arguments, parens, items);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Argument<'a, T>
where
    T: ContextValue<'a>,
{
    pub name: Name<'a, T>,

    #[arbitrary(value = Punctuator::new(":").into())]
    pub colon: Recoverable<Punctuator<'a, T>>,
    pub value: Recoverable<Shared<'a, T, Value<'a, T>>>,
}

node!(
    Shared<'a, T, Argument>,
    visit_argument + post_visit_argument,
    name,
    colon,
    value
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct FragmentSpread<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Punctuator::new("..."))]
    pub dots: Punctuator<'a, T>,
    pub fragment_name: Name<'a, T>,
    pub directives: Option<Directives<'a, T>>,
}

node!(
    Shared<'a, T, FragmentSpread>,
    visit_fragment_spread + post_visit_fragment_spread,
    dots,
    fragment_name,
    directives
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct InlineFragment<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Punctuator::new("..."))]
    pub dots: Punctuator<'a, T>,
    pub type_condition: Option<TypeCondition<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
    pub selection_set: Recoverable<Shared<'a, T, SelectionSet<'a, T>>>,
}

node!(
    InlineFragment,
    visit_inline_fragment + post_visit_inline_fragment,
    dots,
    type_condition,
    directives,
    selection_set
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct FragmentDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Name::new("fragment"))]
    pub fragment: Name<'a, T>,
    pub fragment_name: Recoverable<Name<'a, T>>,
    pub type_condition: Recoverable<TypeCondition<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
    pub selection_set: Recoverable<Shared<'a, T, SelectionSet<'a, T>>>,
}

node!(
    Shared<'a, T, FragmentDefinition>,
    visit_fragment_definition + post_visit_fragment_definition,
    fragment,
    fragment_name,
    type_condition,
    directives,
    selection_set
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct TypeCondition<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Name::new("on"))]
    pub on: Name<'a, T>,
    pub named_type: Recoverable<NamedType<'a, T>>,
}

node!(TypeCondition, visit_type_condition, on, named_type);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum Value<'a, T>
where
    T: ContextValue<'a>,
{
    Variable(Variable<'a, T>),
    IntValue(IntValue<'a, T>),
    FloatValue(FloatValue<'a, T>),
    StringValue(StringValue<'a, T>),
    BooleanValue(BooleanValue<'a, T>),
    NullValue(NullValue<'a, T>),
    EnumValue(EnumValue<'a, T>),
    ListValue(ListValue<'a, T>),
    ObjectValue(ObjectValue<'a, T>),
}

impl<'a, T> Value<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn is_variable(&self) -> bool {
        matches!(self, Value::Variable(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Value::IntValue(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::FloatValue(_))
    }

    pub fn is_float_like(&self) -> bool {
        self.is_int() || self.is_float()
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::StringValue(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::BooleanValue(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::NullValue(_))
    }

    pub fn is_id_like(&self) -> bool {
        self.is_string()
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Value::ListValue(_))
    }
}

impl<'a, T> Value<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn to_json(&self) -> Option<serde_json::Value> {
        match self {
            Value::Variable(_) => None,
            Value::IntValue(value) => Some(serde_json::Value::Number(value.to_i32().ok()?.into())),
            Value::FloatValue(value) => Some(serde_json::Value::Number(
                serde_json::Number::from_f64(value.to_f64().ok()?)?,
            )),
            Value::StringValue(value) => Some(serde_json::Value::String(value.to_string())),
            Value::EnumValue(value) => Some(match value.0.to_string() {
                value if value == "true" => serde_json::Value::Bool(true),
                value if value == "false" => serde_json::Value::Bool(false),
                value => serde_json::Value::String(value),
            }),
            Value::NullValue(_) => Some(serde_json::Value::Null),
            Value::BooleanValue(value) => Some(serde_json::Value::Bool(value.to_bool())),
            Value::ListValue(value) => Some(serde_json::Value::Array(
                value
                    .values
                    .iter()
                    .map(|value| value.to_json())
                    .collect::<Option<_>>()?,
            )),
            Value::ObjectValue(value) => Some(serde_json::Value::Object(
                value
                    .object_fields
                    .iter()
                    .map(|field| Some((field.name.to_string(), field.value.ok()?.to_json()?)))
                    .collect::<Option<_>>()?,
            )),
        }
    }
}

node_enum!(
    Shared<'a, T, Value>,
    visit_value + post_visit_value,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum BooleanValue<'a, T>
where
    T: ContextValue<'a>,
{
    True(#[arbitrary(value = Name::new("true"))] Name<'a, T>),
    False(#[arbitrary(value = Name::new("false"))] Name<'a, T>),
}

impl<'a, T> BooleanValue<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn to_bool(&self) -> bool {
        matches!(self, BooleanValue::True(_))
    }
}

node_enum!(BooleanValue, visit_boolean_value, True, False);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct NullValue<'a, T>(#[arbitrary(value = Name::new("null"))] pub Name<'a, T>)
where
    T: ContextValue<'a>;

node_unit!(NullValue, visit_null_value);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct EnumValue<'a, T>(pub Name<'a, T>)
where
    T: ContextValue<'a>;

node_unit!(EnumValue, visit_enum_value);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ListValue<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("[", "]"))]
    pub brackets: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),

    pub values: List<'a, T, Shared<'a, T, Value<'a, T>>>,
}

node!(
    ListValue,
    visit_list_value + post_visit_list_value,
    brackets,
    values
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ObjectValue<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("{", "}"))]
    pub braces: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),

    pub object_fields: List<'a, T, ObjectField<'a, T>>,
}

node!(ObjectValue, visit_object_value, braces, object_fields);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ObjectField<'a, T>
where
    T: ContextValue<'a>,
{
    pub name: Name<'a, T>,

    #[arbitrary(value = Punctuator::new(":").into())]
    pub colon: Recoverable<Punctuator<'a, T>>,

    pub value: Recoverable<Shared<'a, T, Value<'a, T>>>,
}

node!(
    ObjectField,
    visit_object_field + post_visit_object_field,
    name,
    colon,
    value
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct VariableDefinitions<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("(", ")"))]
    pub parens: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),
    pub variable_definitions: List<'a, T, Shared<'a, T, VariableDefinition<'a, T>>>,
}

node!(
    VariableDefinitions,
    visit_variable_definitions,
    parens,
    variable_definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct VariableDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub variable: Variable<'a, T>,

    #[arbitrary(value = Punctuator::new(":").into())]
    pub colon: Recoverable<Punctuator<'a, T>>,

    pub ty: Recoverable<Shared<'a, T, Type<'a, T>>>,

    pub default_value: Option<DefaultValue<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
}

node!(
    Shared<'a, T, VariableDefinition>,
    visit_variable_definition + post_visit_variable_definition,
    variable,
    colon,
    ty,
    default_value,
    directives
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Variable<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Punctuator::new("$"))]
    pub dollar: Punctuator<'a, T>,
    pub name: Recoverable<Name<'a, T>>,
}

node!(Variable, visit_variable, dollar, name);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct DefaultValue<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Punctuator::new("="))]
    pub eq: Punctuator<'a, T>,

    pub value: Recoverable<Shared<'a, T, Value<'a, T>>>,
}

node!(DefaultValue, visit_default_value, eq, value);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum Type<'a, T>
where
    T: ContextValue<'a>,
{
    Named(NamedType<'a, T>),
    List(ListType<'a, T>),
    NonNull(NonNullType<'a, T>),
}

impl<'a, T> Type<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn name(&self) -> Option<&T> {
        self.named_type().map(|ty| ty.0.as_ref())
    }

    pub fn named_type(&self) -> Option<&NamedType<'a, T>> {
        match self {
            Type::Named(ty) => Some(ty),
            Type::List(ty) => ty.ty.ok().and_then(|ty| ty.named_type()),
            Type::NonNull(ty) => ty.ty.named_type(),
        }
    }

    pub fn is_nullable(&self) -> bool {
        !self.is_required()
    }

    pub fn is_required(&self) -> bool {
        matches!(self, Type::NonNull(_))
    }

    pub fn as_nullable<'b>(this: &'b Shared<'a, T, Type<'a, T>>) -> &'b Shared<'a, T, Type<'a, T>> {
        match this.as_ref() {
            Type::Named(_) | Type::List(_) => this,
            Type::NonNull(ty) => Type::as_nullable(&ty.ty),
        }
    }

    pub fn list_value_type(&self) -> Option<&Shared<'a, T, Type<'a, T>>> {
        match self {
            Type::List(ty) => ty.ty.ok(),
            Type::NonNull(ty) => ty.ty.list_value_type(),
            Type::Named(_) => None,
        }
    }
}

impl<'a, T> Type<'a, T>
where
    T: ContextValue<'a> + Eq,
{
    pub fn is_invariant(&self, other: &Type<'a, T>) -> bool {
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

impl<'a, T> Display for Type<'a, T>
where
    T: ContextValue<'a> + ToString,
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

node_enum!(Shared<'a, T, Type>, visit_type, Named, List, NonNull);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct NamedType<'a, T>(pub Name<'a, T>)
where
    T: ContextValue<'a>;

node_unit!(NamedType, visit_named_type);
node_arc!(NamedType);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ListType<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("[", "]"))]
    pub brackets: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),

    pub ty: Recoverable<Shared<'a, T, Type<'a, T>>>,
}

node!(ListType, visit_list_type, brackets, ty);

#[derive(Clone, Debug)]
pub struct NonNullType<'a, T>
where
    T: ContextValue<'a>,
{
    pub ty: Shared<'a, T, Type<'a, T>>,
    pub bang: Punctuator<'a, T>,
}

node!(NonNullType, visit_non_null_type, ty, bang);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Directives<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(with = arbitrary_at_least_one)]
    pub directives: List<'a, T, Shared<'a, T, Directive<'a, T>>>,
}

node!(Directives, visit_directives, directives);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Directive<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Punctuator::new("@"))]
    pub at: Punctuator<'a, T>,

    pub name: Recoverable<Name<'a, T>>,

    pub arguments: Option<Shared<'a, T, Arguments<'a, T>>>,
}

impl<'a, T> Directive<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn argument(&self, name: &str) -> Option<&Shared<'a, T, Argument<'a, T>>> {
        self.arguments.as_ref().and_then(move |arguments| {
            arguments
                .items
                .iter()
                .find(|arg| arg.name.as_ref().borrow() == name)
        })
    }
}

node!(
    Shared<'a, T, Directive>,
    visit_directive,
    at,
    name,
    arguments
);

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
#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct TypeSystemDocument<'a, T>
where
    T: ContextValue<'a>,
{
    pub definitions: List<'a, T, TypeSystemDefinition<'a, T>>,
}

node!(TypeSystemDocument, visit_type_system_document, definitions);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum TypeSystemDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    SchemaDefinition(SchemaDefinition<'a, T>),
    TypeDefinition(Shared<'a, T, TypeDefinition<'a, T>>),
    DirectiveDefinition(Shared<'a, T, DirectiveDefinition<'a, T>>),

    #[arbitrary(skip)]
    Error(Description<'a, T>),
}

node_enum!(
    TypeSystemDefinition,
    visit_type_system_definition,
    SchemaDefinition,
    TypeDefinition,
    DirectiveDefinition,
    Error
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct TypeSystemExtensionDocument<'a, T>
where
    T: ContextValue<'a>,
{
    pub definitions: List<'a, T, TypeSystemDefinitionOrExtension<'a, T>>,
}

node!(
    TypeSystemExtensionDocument,
    visit_type_system_extension_document,
    definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum TypeSystemDefinitionOrExtension<'a, T>
where
    T: ContextValue<'a>,
{
    TypeSystemDefinition(TypeSystemDefinition<'a, T>),
    TypeSystemExtension(TypeSystemExtension<'a, T>),
}

node_enum!(
    TypeSystemDefinitionOrExtension,
    visit_type_system_definition_or_extension,
    TypeSystemDefinition,
    TypeSystemExtension
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum TypeSystemExtension<'a, T>
where
    T: ContextValue<'a>,
{
    SchemaExtension(SchemaExtension<'a, T>),
    TypeExtension(Shared<'a, T, TypeExtension<'a, T>>),

    #[arbitrary(skip)]
    Error(Name<'a, T>),
}

node_enum!(
    TypeSystemExtension,
    visit_type_system_extension,
    SchemaExtension,
    TypeExtension,
    Error
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct Description<'a, T>(pub StringValue<'a, T>)
where
    T: ContextValue<'a>;

impl<'a, T> ToString for Description<'a, T>
where
    T: ContextValue<'a> + From<&'static str>,
{
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

node_unit!(Description, visit_description);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct SchemaDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("schema"))]
    pub schema: Name<'a, T>,

    pub directives: Option<Directives<'a, T>>,

    pub type_definitions: Recoverable<RootOperationTypeDefinitions<'a, T>>,
}

node!(
    SchemaDefinition,
    visit_schema_definition,
    description,
    schema,
    directives,
    type_definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct RootOperationTypeDefinitions<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("{", "}"))]
    pub braces: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),

    #[arbitrary(with = arbitrary_present_at_least_one)]
    pub definitions: Recoverable<List<'a, T, RootOperationTypeDefinition<'a, T>>>,
}

node!(
    RootOperationTypeDefinitions,
    visit_root_operation_type_definitions,
    braces,
    definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct RootOperationTypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub operation_type: OperationType<'a, T>,

    #[arbitrary(value = Punctuator::new(":").into())]
    pub colon: Recoverable<Punctuator<'a, T>>,

    pub named_type: Recoverable<NamedType<'a, T>>,
}

node!(
    RootOperationTypeDefinition,
    visit_root_operation_type_definition,
    operation_type,
    colon,
    named_type
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct SchemaExtension<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = (Name::new("extend"), Name::new("schema")))]
    pub extend_schema: (Name<'a, T>, Name<'a, T>),
    pub directives: Option<Directives<'a, T>>,
    pub type_definitions: Option<RootOperationTypeDefinitions<'a, T>>,
}

node!(
    SchemaExtension,
    visit_schema_extension,
    extend_schema,
    directives,
    type_definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum TypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    ScalarTypeDefinition(ScalarTypeDefinition<'a, T>),
    ObjectTypeDefinition(ObjectTypeDefinition<'a, T>),
    InterfaceTypeDefinition(InterfaceTypeDefinition<'a, T>),
    UnionTypeDefinition(UnionTypeDefinition<'a, T>),
    EnumTypeDefinition(EnumTypeDefinition<'a, T>),
    InputObjectTypeDefinition(InputObjectTypeDefinition<'a, T>),
}

node_enum!(
    Shared<'a, T, TypeDefinition>,
    visit_type_definition,
    ScalarTypeDefinition,
    ObjectTypeDefinition,
    InterfaceTypeDefinition,
    UnionTypeDefinition,
    EnumTypeDefinition,
    InputObjectTypeDefinition
);

impl<'a, T> TypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn keyword(&self) -> &Name<'a, T> {
        match self {
            TypeDefinition::ScalarTypeDefinition(definition) => &definition.scalar,
            TypeDefinition::ObjectTypeDefinition(definition) => &definition.ty,
            TypeDefinition::InterfaceTypeDefinition(definition) => &definition.interface,
            TypeDefinition::UnionTypeDefinition(definition) => &definition.union_kw,
            TypeDefinition::EnumTypeDefinition(definition) => &definition.enum_kw,
            TypeDefinition::InputObjectTypeDefinition(definition) => &definition.input,
        }
    }

    pub fn description(&self) -> Option<&Description<'a, T>> {
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

    pub fn name(&self) -> &Recoverable<Name<'a, T>> {
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

    pub fn is_enum(&self) -> bool {
        matches!(self, TypeDefinition::EnumTypeDefinition(_))
    }

    pub fn is_interface(&self) -> bool {
        matches!(self, TypeDefinition::InterfaceTypeDefinition(_))
    }

    pub fn is_input_object_type(&self) -> bool {
        matches!(self, TypeDefinition::InputObjectTypeDefinition(_))
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self, TypeDefinition::ScalarTypeDefinition(_))
    }

    pub fn is_scalar_like(&self) -> bool {
        self.is_scalar() || self.is_enum()
    }

    pub fn is_union(&self) -> bool {
        matches!(self, TypeDefinition::UnionTypeDefinition(_))
    }

    pub fn is_composite(&self) -> bool {
        self.is_object_type() || self.is_interface() || self.is_union()
    }

    pub fn is_object_type(&self) -> bool {
        matches!(self, TypeDefinition::ObjectTypeDefinition(_))
    }

    pub fn directives(&self) -> Option<&Directives<'a, T>> {
        match self {
            TypeDefinition::EnumTypeDefinition(definition) => definition.directives.as_ref(),
            TypeDefinition::InputObjectTypeDefinition(definition) => definition.directives.as_ref(),
            TypeDefinition::InterfaceTypeDefinition(definition) => definition.directives.as_ref(),
            TypeDefinition::ObjectTypeDefinition(definition) => definition.directives.as_ref(),
            TypeDefinition::ScalarTypeDefinition(definition) => definition.directives.as_ref(),
            TypeDefinition::UnionTypeDefinition(definition) => definition.directives.as_ref(),
        }
    }

    pub fn implements_interfaces(&self) -> Option<&ImplementsInterfaces<'a, T>> {
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

    pub fn fields_definition(&self) -> Option<&FieldsDefinition<'a, T>> {
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum TypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    ScalarTypeExtension(ScalarTypeExtension<'a, T>),
    ObjectTypeExtension(ObjectTypeExtension<'a, T>),
    InterfaceTypeExtension(InterfaceTypeExtension<'a, T>),
    UnionTypeExtension(UnionTypeExtension<'a, T>),
    EnumTypeExtension(EnumTypeExtension<'a, T>),
    InputObjectTypeExtension(InputObjectTypeExtension<'a, T>),
}

node_enum!(
    Shared<'a, T, TypeExtension>,
    visit_type_extension,
    ScalarTypeExtension,
    ObjectTypeExtension,
    InterfaceTypeExtension,
    UnionTypeExtension,
    EnumTypeExtension,
    InputObjectTypeExtension
);

impl<'a, T> TypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn keyword(&self) -> &Name<'a, T> {
        match self {
            TypeExtension::ScalarTypeExtension(extension) => &extension.extend_scalar.1,
            TypeExtension::ObjectTypeExtension(extension) => &extension.extend_type.1,
            TypeExtension::InterfaceTypeExtension(extension) => &extension.extend_interface.1,
            TypeExtension::UnionTypeExtension(extension) => &extension.extend_union.1,
            TypeExtension::EnumTypeExtension(extension) => &extension.extend_enum.1,
            TypeExtension::InputObjectTypeExtension(extension) => &extension.extend_input.1,
        }
    }

    pub fn name(&self) -> Option<&T> {
        match self {
            TypeExtension::ScalarTypeExtension(extension) => &extension.name,
            TypeExtension::ObjectTypeExtension(extension) => &extension.name,
            TypeExtension::InterfaceTypeExtension(extension) => &extension.name,
            TypeExtension::UnionTypeExtension(extension) => &extension.name,
            TypeExtension::EnumTypeExtension(extension) => &extension.name,
            TypeExtension::InputObjectTypeExtension(extension) => &extension.name,
        }
        .ok()
        .map(|name| name.0.as_ref())
    }

    pub fn directives(&self) -> Option<&Directives<'a, T>> {
        match self {
            TypeExtension::EnumTypeExtension(extension) => extension.directives.as_ref(),
            TypeExtension::InputObjectTypeExtension(extension) => extension.directives.as_ref(),
            TypeExtension::InterfaceTypeExtension(extension) => extension.directives.as_ref(),
            TypeExtension::ObjectTypeExtension(extension) => extension.directives.as_ref(),
            TypeExtension::ScalarTypeExtension(extension) => extension.directives.ok(),
            TypeExtension::UnionTypeExtension(extension) => extension.directives.as_ref(),
        }
    }

    pub fn implements_interfaces(&self) -> Option<&ImplementsInterfaces<'a, T>> {
        match self {
            TypeExtension::InterfaceTypeExtension(extension) => {
                extension.implements_interfaces.as_ref()
            }
            TypeExtension::ObjectTypeExtension(extension) => {
                extension.implements_interfaces.as_ref()
            }
            _ => None,
        }
    }

    pub fn fields_definition(&self) -> Option<&FieldsDefinition<'a, T>> {
        match self {
            TypeExtension::InterfaceTypeExtension(extension) => {
                extension.fields_definition.as_ref()
            }
            TypeExtension::ObjectTypeExtension(extension) => extension.fields_definition.as_ref(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ScalarTypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("scalar"))]
    pub scalar: Name<'a, T>,

    pub name: Recoverable<Name<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
}

node!(
    ScalarTypeDefinition,
    visit_scalar_type_definition,
    scalar,
    name,
    directives
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ScalarTypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = (Name::new("extend"), Name::new("scalar")))]
    pub extend_scalar: (Name<'a, T>, Name<'a, T>),

    pub name: Recoverable<NamedType<'a, T>>,

    pub directives: Recoverable<Directives<'a, T>>,
}

node!(
    ScalarTypeExtension,
    visit_scalar_type_extension,
    extend_scalar,
    name,
    directives
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ObjectTypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("type"))]
    pub ty: Name<'a, T>,

    pub name: Recoverable<Name<'a, T>>,

    pub implements_interfaces: Option<ImplementsInterfaces<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
    pub fields_definition: Option<FieldsDefinition<'a, T>>,
}

impl<'a, T> ObjectTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Eq,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ImplementsInterfaces<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Name::new("implements"))]
    pub implements: Name<'a, T>,

    #[arbitrary(value = None)]
    pub ampersand: Option<Punctuator<'a, T>>,

    pub first: Recoverable<Shared<'a, T, NamedType<'a, T>>>,

    #[arbitrary(value = todo!())]
    pub types: List<
        'a,
        T,
        (
            Punctuator<'a, T>,
            Recoverable<Shared<'a, T, NamedType<'a, T>>>,
        ),
    >,
}

impl<'a, T> ImplementsInterfaces<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn named_types(&self) -> impl Iterator<Item = &Shared<'a, T, NamedType<'a, T>>> {
        self.first
            .ok()
            .into_iter()
            .chain(self.types.iter().flat_map(|(_, ty)| ty.ok()))
    }

    // pub fn types(&self) -> impl Iterator<Item = &T> {
    // self.named_types().map(|ty| ty.0.as_ref())
    // }
}

impl<'a, T> ImplementsInterfaces<'a, T>
where
    T: ContextValue<'a> + Eq,
{
    pub fn implements_interface(&self, name: &T) -> bool {
        self.named_types().any(|ty| ty.0.as_ref() == name)
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct FieldsDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("{", "}"))]
    pub braces: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),
    pub definitions: List<'a, T, Shared<'a, T, FieldDefinition<'a, T>>>,
}

impl<'a, T> FieldsDefinition<'a, T>
where
    T: ContextValue<'a> + Eq,
{
    pub fn field(&self, name: &T) -> Option<&FieldDefinition<'a, T>> {
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct FieldDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,
    pub name: Name<'a, T>,
    pub arguments_definition: Option<Shared<'a, T, ArgumentsDefinition<'a, T>>>,

    #[arbitrary(value = Punctuator::new(":").into())]
    pub colon: Recoverable<Punctuator<'a, T>>,

    pub ty: Recoverable<Shared<'a, T, Type<'a, T>>>,
    pub directives: Option<Directives<'a, T>>,
}

node!(
    Shared<'a, T, FieldDefinition>,
    visit_field_definition,
    description,
    name,
    arguments_definition,
    colon,
    ty,
    directives
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ArgumentsDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("(", ")"))]
    pub parens: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),

    pub definitions: List<'a, T, Shared<'a, T, InputValueDefinition<'a, T>>>,
}

impl<'a, T> ArgumentsDefinition<'a, T>
where
    T: ContextValue<'a> + Eq,
{
    pub fn argument(&self, name: &T) -> Option<&Shared<'a, T, InputValueDefinition<'a, T>>> {
        self.definitions
            .iter()
            .find(|def| def.name.as_ref() == name)
    }
}

node!(
    Shared<'a, T, ArgumentsDefinition>,
    visit_arguments_definition,
    parens,
    definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct InputValueDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,
    pub name: Name<'a, T>,

    #[arbitrary(value = Punctuator::new(":").into())]
    pub colon: Recoverable<Punctuator<'a, T>>,

    pub ty: Recoverable<Shared<'a, T, Type<'a, T>>>,

    pub default_value: Option<DefaultValue<'a, T>>,

    pub directives: Option<Directives<'a, T>>,
}

impl<'a, T> InputValueDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn is_required(&self) -> bool {
        let ty = match self.ty.ok() {
            Some(ty) if ty.is_required() => ty,
            Some(_) | None => return false,
        };

        match self
            .default_value
            .as_ref()
            .and_then(|value| value.value.ok())
        {
            Some(value) => ty.is_required() && value.is_null(),
            None => ty.is_required(),
        }
    }
}

node!(
    Shared<'a, T, InputValueDefinition>,
    visit_input_value_definition + post_visit_input_value_definition,
    description,
    name,
    colon,
    ty,
    default_value,
    directives
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct ObjectTypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = (Name::new("extend"), Name::new("type")))]
    pub extend_type: (Name<'a, T>, Name<'a, T>),

    pub name: Recoverable<NamedType<'a, T>>,

    pub implements_interfaces: Option<ImplementsInterfaces<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
    pub fields_definition: Option<FieldsDefinition<'a, T>>,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct InterfaceTypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("interface"))]
    pub interface: Name<'a, T>,

    pub name: Recoverable<Name<'a, T>>,

    pub implements_interfaces: Option<ImplementsInterfaces<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
    pub fields_definition: Option<FieldsDefinition<'a, T>>,
}

impl<'a, T> InterfaceTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Eq,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct InterfaceTypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = (Name::new("extend"), Name::new("interface")))]
    pub extend_interface: (Name<'a, T>, Name<'a, T>),

    pub name: Recoverable<NamedType<'a, T>>,

    pub implements_interfaces: Option<ImplementsInterfaces<'a, T>>,
    pub directives: Option<Directives<'a, T>>,
    pub fields_definition: Option<FieldsDefinition<'a, T>>,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct UnionTypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("union"))]
    pub union_kw: Name<'a, T>,

    pub name: Recoverable<Name<'a, T>>,

    pub directives: Option<Directives<'a, T>>,
    pub member_types: Option<UnionMemberTypes<'a, T>>,
}

impl<'a, T> UnionTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Eq,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct UnionMemberTypes<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Punctuator::new("="))]
    pub eq: Punctuator<'a, T>,

    #[arbitrary(value = None)]
    pub pipe: Option<Punctuator<'a, T>>,

    pub first: Recoverable<Shared<'a, T, NamedType<'a, T>>>,

    #[arbitrary(value = todo!())]
    pub types: List<
        'a,
        T,
        (
            Punctuator<'a, T>,
            Recoverable<Shared<'a, T, NamedType<'a, T>>>,
        ),
    >,
}

impl<'a, T> UnionMemberTypes<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn named_types(&self) -> impl Iterator<Item = &Shared<'a, T, NamedType<'a, T>>> {
        self.first
            .ok()
            .into_iter()
            .chain(self.types.iter().flat_map(|(_, ty)| ty.ok()))
    }

    // pub fn types(&self) -> impl Iterator<Item = &T> {
    //     self.named_types().map(|ty| ty.0.as_ref())
    // }
}

impl<'a, T> UnionMemberTypes<'a, T>
where
    T: ContextValue<'a> + Eq,
{
    pub fn includes_member_type(&self, name: &T) -> bool {
        self.named_types().any(|ty| ty.0.as_ref() == name)
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct UnionTypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = (Name::new("extend"), Name::new("union")))]
    pub extend_union: (Name<'a, T>, Name<'a, T>),

    pub name: Recoverable<NamedType<'a, T>>,

    pub directives: Option<Directives<'a, T>>,
    pub member_types: Option<UnionMemberTypes<'a, T>>,
}

node!(
    UnionTypeExtension,
    visit_union_type_extension,
    extend_union,
    name,
    directives,
    member_types
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct EnumTypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("enum"))]
    pub enum_kw: Name<'a, T>,

    pub name: Recoverable<Name<'a, T>>,

    pub directives: Option<Directives<'a, T>>,
    pub values_definition: Option<EnumValuesDefinition<'a, T>>,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct EnumValuesDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("{", "}"))]
    pub braces: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),
    pub definitions: List<'a, T, Shared<'a, T, EnumValueDefinition<'a, T>>>,
}

node!(
    EnumValuesDefinition,
    visit_enum_values_definition,
    braces,
    definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct EnumValueDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,
    pub enum_value: EnumValue<'a, T>,
    pub directives: Option<Directives<'a, T>>,
}

node!(
    Shared<'a, T, EnumValueDefinition>,
    visit_enum_value_definition,
    description,
    enum_value,
    directives
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct EnumTypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = (Name::new("extend"), Name::new("enum")))]
    pub extend_enum: (Name<'a, T>, Name<'a, T>),

    pub name: Recoverable<NamedType<'a, T>>,

    pub directives: Option<Directives<'a, T>>,
    pub values_definition: Option<EnumValuesDefinition<'a, T>>,
}

node!(
    EnumTypeExtension,
    visit_enum_type_extension,
    name,
    directives,
    values_definition
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct InputObjectTypeDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("input"))]
    pub input: Name<'a, T>,

    pub name: Recoverable<Name<'a, T>>,

    pub directives: Option<Directives<'a, T>>,
    pub fields_definition: Option<InputFieldsDefinition<'a, T>>,
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct InputFieldsDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = arbitrary_punctuators("{", "}"))]
    pub braces: (Punctuator<'a, T>, Recoverable<Punctuator<'a, T>>),

    pub definitions: List<'a, T, Shared<'a, T, InputValueDefinition<'a, T>>>,
}

node!(
    InputFieldsDefinition,
    visit_input_fields_definition,
    braces,
    definitions
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct InputObjectTypeExtension<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = (Name::new("extend"), Name::new("input")))]
    pub extend_input: (Name<'a, T>, Name<'a, T>),

    pub name: Recoverable<NamedType<'a, T>>,

    pub directives: Option<Directives<'a, T>>,
    pub fields_definition: Option<InputFieldsDefinition<'a, T>>,
}

node!(
    InputObjectTypeExtension,
    visit_input_object_type_extension,
    extend_input,
    name,
    directives,
    fields_definition
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct DirectiveDefinition<'a, T>
where
    T: ContextValue<'a>,
{
    pub description: Option<Description<'a, T>>,

    #[arbitrary(value = Name::new("directive"))]
    pub directive: Name<'a, T>,

    #[arbitrary(value = Punctuator::new("@").into())]
    pub at: Recoverable<Punctuator<'a, T>>,
    pub name: Recoverable<Name<'a, T>>,

    pub arguments_definition: Option<Shared<'a, T, ArgumentsDefinition<'a, T>>>,

    #[arbitrary(value = Some(Name::new("repeatable")))]
    pub repeatable: Option<Name<'a, T>>,
    pub locations: Recoverable<DirectiveLocations<'a, T>>,
}

node!(
    Shared<'a, T, DirectiveDefinition>,
    visit_directive_definition,
    description,
    directive,
    at,
    name,
    arguments_definition,
    repeatable,
    locations
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub struct DirectiveLocations<'a, T>
where
    T: ContextValue<'a>,
{
    #[arbitrary(value = Name::new("on"))]
    pub on: Name<'a, T>,

    #[arbitrary(value = None)]
    pub pipe: Option<Punctuator<'a, T>>,

    pub first: Recoverable<DirectiveLocation<'a, T>>,

    #[arbitrary(value = todo!())]
    pub locations: List<'a, T, (Punctuator<'a, T>, Recoverable<DirectiveLocation<'a, T>>)>,
}

impl<'a, T> DirectiveLocations<'a, T>
where
    T: ContextValue<'a>,
{
    pub fn locations(&self) -> impl Iterator<Item = &DirectiveLocation<'a, T>> {
        self.first.ok().into_iter().chain(
            self.locations
                .iter()
                .flat_map(|(_, location)| location.ok()),
        )
    }
}

node!(
    DirectiveLocations,
    visit_directive_locations,
    on,
    pipe,
    first,
    locations
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum DirectiveLocation<'a, T>
where
    T: ContextValue<'a>,
{
    ExecutableDirectiveLocation(ExecutableDirectiveLocation<'a, T>),
    TypeSystemDirectiveLocation(TypeSystemDirectiveLocation<'a, T>),
}

node_enum!(
    DirectiveLocation,
    visit_directive_location,
    ExecutableDirectiveLocation,
    TypeSystemDirectiveLocation
);

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum ExecutableDirectiveLocation<'a, T>
where
    T: ContextValue<'a>,
{
    Query(#[arbitrary(value = Name::new("QUERY"))] Name<'a, T>),
    Mutation(#[arbitrary(value = Name::new("MUTATION"))] Name<'a, T>),
    Subscription(#[arbitrary(value = Name::new("SUBSCRIPTION"))] Name<'a, T>),
    Field(#[arbitrary(value = Name::new("FIELD"))] Name<'a, T>),
    FragmentDefinition(#[arbitrary(value = Name::new("FRAGMENT_DEFINITION"))] Name<'a, T>),
    FragmentSpread(#[arbitrary(value = Name::new("FRAGMENT_SPREAD"))] Name<'a, T>),
    InlineFragment(#[arbitrary(value = Name::new("INLINE_FRAGMENT"))] Name<'a, T>),
    VariableDefinition(#[arbitrary(value = Name::new("VARIABLE_DEFINITION"))] Name<'a, T>),
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

#[derive(Clone, Debug, Arbitrary)]
#[arbitrary(bound = "T: From<&'static str> + 'a")]
pub enum TypeSystemDirectiveLocation<'a, T>
where
    T: ContextValue<'a>,
{
    Schema(#[arbitrary(value = Name::new("SCHEMA"))] Name<'a, T>),
    Scalar(#[arbitrary(value = Name::new("SCALAR"))] Name<'a, T>),
    Object(#[arbitrary(value = Name::new("OBJECT"))] Name<'a, T>),
    FieldDefinition(#[arbitrary(value = Name::new("FIELD_DEFINITION"))] Name<'a, T>),
    ArgumentDefinition(#[arbitrary(value = Name::new("ARGUMENT_DEFINITION"))] Name<'a, T>),
    Interface(#[arbitrary(value = Name::new("INTERFACE"))] Name<'a, T>),
    Union(#[arbitrary(value = Name::new("UNION"))] Name<'a, T>),
    Enum(#[arbitrary(value = Name::new("ENUM"))] Name<'a, T>),
    EnumValue(#[arbitrary(value = Name::new("ENUM_VALUE"))] Name<'a, T>),
    InputObject(#[arbitrary(value = Name::new("INPUT_OBJECT"))] Name<'a, T>),
    InputFieldDefinition(#[arbitrary(value = Name::new("INPUT_FIELD_DEFINITION"))] Name<'a, T>),
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
