use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(rename = "__schema")]
    pub schema: Schema,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(rename = "queryType")]
    pub query_type: Type,

    #[serde(rename = "mutationType")]
    pub mutation_type: Option<Type>,

    #[serde(rename = "subscriptionType")]
    pub subscription_type: Option<Type>,

    pub types: Vec<Type>,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub name: Option<String>,
    pub description: Option<String>,
    pub fields: Option<Vec<Field>>,
    pub interfaces: Option<Vec<Type>>,

    #[serde(rename = "possibleTypes")]
    pub possible_types: Option<Vec<Type>>,

    #[serde(rename = "enumValues")]
    pub enum_values: Option<Vec<EnumValue>>,

    #[serde(rename = "inputFields")]
    pub input_fields: Option<Vec<InputValue>>,

    #[serde(rename = "ofType")]
    pub of_type: Option<Box<Type>>,

    #[serde(rename = "specifiedByURL")]
    pub specified_by_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
    List,
    NonNull,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<InputValue>,

    #[serde(rename = "type")]
    pub ty: Type,

    #[serde(rename = "isDeprecated")]
    pub is_deprecated: bool,

    #[serde(rename = "deprecationReason")]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InputValue {
    pub name: String,
    pub description: Option<String>,

    #[serde(rename = "type")]
    pub ty: Type,

    #[serde(rename = "defaultValue")]
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,

    #[serde(rename = "isDeprecated")]
    pub is_deprecated: bool,

    #[serde(rename = "deprecationReason")]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Directive {
    pub name: String,
    pub description: Option<String>,
    pub locations: Vec<DirectiveLocation>,
    pub args: Vec<InputValue>,

    #[serde(default, rename = "isRepeatable")]
    pub is_repeatable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
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
    InputFieldDefinition,
}
