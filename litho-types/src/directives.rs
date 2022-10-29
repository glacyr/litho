use litho_language::ast::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectiveLocationKind {
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

impl ToString for DirectiveLocationKind {
    fn to_string(&self) -> String {
        match self {
            DirectiveLocationKind::Query => "QUERY",
            DirectiveLocationKind::Mutation => "MUTATION",
            DirectiveLocationKind::Subscription => "SUBSCRIPTION",
            DirectiveLocationKind::Field => "FIELD",
            DirectiveLocationKind::FragmentDefinition => "FRAGMENT_DEFINITION",
            DirectiveLocationKind::FragmentSpread => "FRAGMENT_SPREAD",
            DirectiveLocationKind::InlineFragment => "INLINE_FRAGMENT",
            DirectiveLocationKind::VariableDefinition => "VARIABLE_DEFINITION",
            DirectiveLocationKind::Schema => "SCHEMA",
            DirectiveLocationKind::Scalar => "SCALAR",
            DirectiveLocationKind::Object => "OBJECT",
            DirectiveLocationKind::FieldDefinition => "FIELD_DEFINITION",
            DirectiveLocationKind::ArgumentDefinition => "ARGUMENT_DEFINITION",
            DirectiveLocationKind::Interface => "INTERFACE",
            DirectiveLocationKind::Union => "UNION",
            DirectiveLocationKind::Enum => "ENUM",
            DirectiveLocationKind::EnumValue => "ENUM_VALUE",
            DirectiveLocationKind::InputObject => "INPUT_OBJECT",
            DirectiveLocationKind::InputFieldDefinition => "INPUT_FIELD_DEFINITION",
        }
        .to_owned()
    }
}

impl<T> From<&DirectiveLocation<T>> for DirectiveLocationKind {
    fn from(location: &DirectiveLocation<T>) -> Self {
        match location {
            DirectiveLocation::ExecutableDirectiveLocation(location) => location.into(),
            DirectiveLocation::TypeSystemDirectiveLocation(location) => location.into(),
        }
    }
}

impl<T> From<&ExecutableDirectiveLocation<T>> for DirectiveLocationKind {
    fn from(location: &ExecutableDirectiveLocation<T>) -> Self {
        match location {
            ExecutableDirectiveLocation::Query(_) => DirectiveLocationKind::Query,
            ExecutableDirectiveLocation::Mutation(_) => DirectiveLocationKind::Mutation,
            ExecutableDirectiveLocation::Subscription(_) => DirectiveLocationKind::Subscription,
            ExecutableDirectiveLocation::Field(_) => DirectiveLocationKind::Field,
            ExecutableDirectiveLocation::FragmentDefinition(_) => {
                DirectiveLocationKind::FragmentDefinition
            }
            ExecutableDirectiveLocation::FragmentSpread(_) => DirectiveLocationKind::FragmentSpread,
            ExecutableDirectiveLocation::InlineFragment(_) => DirectiveLocationKind::InlineFragment,
            ExecutableDirectiveLocation::VariableDefinition(_) => {
                DirectiveLocationKind::VariableDefinition
            }
        }
    }
}

impl<T> From<&TypeSystemDirectiveLocation<T>> for DirectiveLocationKind {
    fn from(location: &TypeSystemDirectiveLocation<T>) -> Self {
        match location {
            TypeSystemDirectiveLocation::Schema(_) => DirectiveLocationKind::Schema,
            TypeSystemDirectiveLocation::Scalar(_) => DirectiveLocationKind::Scalar,
            TypeSystemDirectiveLocation::Object(_) => DirectiveLocationKind::Object,
            TypeSystemDirectiveLocation::FieldDefinition(_) => {
                DirectiveLocationKind::FieldDefinition
            }
            TypeSystemDirectiveLocation::ArgumentDefinition(_) => {
                DirectiveLocationKind::ArgumentDefinition
            }
            TypeSystemDirectiveLocation::Interface(_) => DirectiveLocationKind::Interface,
            TypeSystemDirectiveLocation::Union(_) => DirectiveLocationKind::Union,
            TypeSystemDirectiveLocation::Enum(_) => DirectiveLocationKind::Enum,
            TypeSystemDirectiveLocation::EnumValue(_) => DirectiveLocationKind::EnumValue,
            TypeSystemDirectiveLocation::InputObject(_) => DirectiveLocationKind::InputObject,
            TypeSystemDirectiveLocation::InputFieldDefinition(_) => {
                DirectiveLocationKind::InputFieldDefinition
            }
        }
    }
}

pub trait DirectiveTarget<T> {
    fn directives(&self) -> Option<&Directives<T>>;
    fn valid_location(&self) -> DirectiveLocationKind;
}

impl<T> DirectiveTarget<T> for OperationDefinition<T> {
    fn directives(&self) -> Option<&Directives<T>> {
        self.directives.as_ref()
    }

    fn valid_location(&self) -> DirectiveLocationKind {
        match self.ty.as_ref() {
            Some(OperationType::Query(_)) | None => DirectiveLocationKind::Query,
            Some(OperationType::Mutation(_)) => DirectiveLocationKind::Mutation,
            Some(OperationType::Subscription(_)) => DirectiveLocationKind::Subscription,
        }
    }
}

impl<T> DirectiveTarget<T> for ScalarTypeExtension<T> {
    fn directives(&self) -> Option<&Directives<T>> {
        self.directives.ok()
    }

    fn valid_location(&self) -> DirectiveLocationKind {
        DirectiveLocationKind::Scalar
    }
}

macro_rules! target {
    ($name:ident) => {
        target!($name, $name);
    };
    ($name:ident, $enum:ident) => {
        impl<T> DirectiveTarget<T> for $name<T> {
            fn directives(&self) -> Option<&Directives<T>> {
                self.directives.as_ref()
            }

            fn valid_location(&self) -> DirectiveLocationKind {
                DirectiveLocationKind::$enum
            }
        }
    };
}

target!(Field);
target!(FragmentSpread);
target!(FragmentDefinition);
target!(InlineFragment);
target!(VariableDefinition);
target!(SchemaDefinition, Schema);
target!(SchemaExtension, Schema);
target!(ScalarTypeDefinition, Scalar);
target!(ObjectTypeDefinition, Object);
target!(FieldDefinition, FieldDefinition);
target!(InputValueDefinition, ArgumentDefinition);
target!(ObjectTypeExtension, Object);
target!(InterfaceTypeDefinition, Interface);
target!(InterfaceTypeExtension, Interface);
target!(UnionTypeDefinition, Union);
target!(UnionTypeExtension, Union);
target!(EnumTypeDefinition, Enum);
target!(EnumValueDefinition, EnumValue);
target!(EnumTypeExtension, Enum);
target!(InputObjectTypeDefinition, InputObject);
target!(InputObjectTypeExtension, InputObject);
