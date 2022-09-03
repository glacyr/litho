use std::iter::{empty, once};

use graphql_parser::schema::{
    Definition, DirectiveDefinition, Document, EnumType, Field, InputValue, Text, Type,
    TypeDefinition,
};

pub trait DirectiveDefinitionExt<'a, T>
where
    T: Text<'a>,
{
    fn argument(&self, name: &str) -> Option<&InputValue<'a, T>>;
}

impl<'a, T> DirectiveDefinitionExt<'a, T> for DirectiveDefinition<'a, T>
where
    T: Text<'a>,
{
    fn argument(&self, name: &str) -> Option<&InputValue<'a, T>> {
        self.arguments
            .iter()
            .find(|value| value.name.as_ref() == name)
    }
}

pub trait DocumentExt<'a, T>
where
    T: Text<'a>,
{
    fn type_definitions(&self) -> Box<dyn Iterator<Item = &TypeDefinition<'a, T>> + '_>;

    fn type_definition<S>(&self, name: S) -> Option<&TypeDefinition<'a, T>>
    where
        S: AsRef<str>;

    fn directive_definitions(&self) -> Box<dyn Iterator<Item = &DirectiveDefinition<'a, T>> + '_>;

    fn directive_definition<S>(&self, name: S) -> Option<&DirectiveDefinition<'a, T>>
    where
        S: AsRef<str>;

    fn possible_types<'b>(&'b self, name: &'b str) -> Box<dyn Iterator<Item = &'b str> + 'b>;

    fn types_implementing_interface<'b>(
        &'b self,
        name: &'b str,
    ) -> Box<dyn Iterator<Item = &'b str> + '_>;
}

impl<'a, T> DocumentExt<'a, T> for Document<'a, T>
where
    T: Text<'a>,
{
    fn type_definitions(&self) -> Box<dyn Iterator<Item = &TypeDefinition<'a, T>> + '_> {
        Box::new(
            self.definitions
                .iter()
                .flat_map(|definition| match definition {
                    Definition::TypeDefinition(definition) => Some(definition),
                    _ => None,
                }),
        )
    }

    fn type_definition<S>(&self, name: S) -> Option<&TypeDefinition<'a, T>>
    where
        S: AsRef<str>,
    {
        self.type_definitions()
            .find(|definition| definition.name().as_ref() == name.as_ref())
    }

    fn directive_definitions(&self) -> Box<dyn Iterator<Item = &DirectiveDefinition<'a, T>> + '_> {
        Box::new(
            self.definitions
                .iter()
                .flat_map(|definition| match definition {
                    Definition::DirectiveDefinition(definition) => Some(definition),
                    _ => None,
                }),
        )
    }

    fn directive_definition<S>(&self, name: S) -> Option<&DirectiveDefinition<'a, T>>
    where
        S: AsRef<str>,
    {
        self.directive_definitions()
            .find(|definition| definition.name.as_ref() == name.as_ref())
    }

    fn possible_types<'b>(&'b self, name: &'b str) -> Box<dyn Iterator<Item = &'b str> + 'b> {
        let definition = match self.type_definition(name) {
            Some(definition) => definition,
            None => return Box::new(empty()),
        };

        match definition {
            TypeDefinition::Enum(_)
            | TypeDefinition::InputObject(_)
            | TypeDefinition::Scalar(_) => Box::new(empty()),
            TypeDefinition::Interface(_) => self.types_implementing_interface(name),
            TypeDefinition::Object(_) => Box::new(once(name)),
            TypeDefinition::Union(definition) => {
                Box::new(definition.types.iter().map(AsRef::as_ref))
            }
        }
    }

    fn types_implementing_interface<'b>(
        &'b self,
        name: &'b str,
    ) -> Box<dyn Iterator<Item = &'b str> + '_> {
        Box::new(
            self.definitions
                .iter()
                .filter_map(move |definition| match definition {
                    Definition::TypeDefinition(definition)
                        if definition.implements_interface(&name) =>
                    {
                        Some(definition.name().as_ref())
                    }
                    _ => None,
                }),
        )
    }
}

pub trait EnumTypeExt {
    fn has_value(&self, name: &str) -> bool;
}

impl<'a, T> EnumTypeExt for EnumType<'a, T>
where
    T: Text<'a>,
{
    fn has_value(&self, name: &str) -> bool {
        self.values
            .iter()
            .find(|value| value.name.as_ref() == name)
            .is_some()
    }
}

pub trait FieldDefinitionExt<'a, T>
where
    T: Text<'a>,
{
    fn argument(&self, name: &str) -> Option<&InputValue<'a, T>>;
}

impl<'a, T> FieldDefinitionExt<'a, T> for Field<'a, T>
where
    T: Text<'a>,
{
    fn argument(&self, name: &str) -> Option<&InputValue<'a, T>> {
        self.arguments
            .iter()
            .find(|input| input.name.as_ref() == name)
    }
}

pub trait TypeExt<'a, T>
where
    T: Text<'a>,
{
    fn name(&self) -> &T::Value;

    fn is_required(&self) -> bool;
}

impl<'a, T> TypeExt<'a, T> for Type<'a, T>
where
    T: Text<'a>,
{
    fn name(&self) -> &T::Value {
        match self {
            Type::ListType(ty) | Type::NonNullType(ty) => ty.name(),
            Type::NamedType(name) => name,
        }
    }

    fn is_required(&self) -> bool {
        match self {
            Type::NonNullType(_) => true,
            Type::ListType(_) | Type::NamedType(_) => false,
        }
    }
}

pub trait TypeDefinitionExt<'a, T>
where
    T: Text<'a>,
{
    fn name(&self) -> &T::Value;

    fn fields(&self) -> Box<dyn Iterator<Item = &Field<'a, T>> + '_>;

    fn field(&self, name: &T::Value) -> Option<&Field<'a, T>>;

    fn input_values(&self) -> Box<dyn Iterator<Item = &InputValue<'a, T>> + '_>;

    fn input_value(&self, name: &T::Value) -> Option<&InputValue<'a, T>>;

    fn is_composite(&self) -> bool;

    fn implements_interfaces(&self) -> Box<dyn Iterator<Item = &str> + '_>;

    fn implements_interface(&self, name: &str) -> bool;
}

impl<'a, T> TypeDefinitionExt<'a, T> for TypeDefinition<'a, T>
where
    T: Text<'a>,
{
    fn name(&self) -> &T::Value {
        match self {
            TypeDefinition::Scalar(ty) => &ty.name,
            TypeDefinition::Object(ty) => &ty.name,
            TypeDefinition::Interface(ty) => &ty.name,
            TypeDefinition::Union(ty) => &ty.name,
            TypeDefinition::Enum(ty) => &ty.name,
            TypeDefinition::InputObject(ty) => &ty.name,
        }
    }

    fn fields(&self) -> Box<dyn Iterator<Item = &Field<'a, T>> + '_> {
        match self {
            TypeDefinition::Enum(_)
            | TypeDefinition::Scalar(_)
            | TypeDefinition::InputObject(_)
            | TypeDefinition::Union(_) => Box::new(empty()),
            TypeDefinition::Object(ty) => Box::new(ty.fields.iter()),
            TypeDefinition::Interface(ty) => Box::new(ty.fields.iter()),
        }
    }

    fn field(&self, name: &T::Value) -> Option<&Field<'a, T>> {
        self.fields().find(|field| &field.name == name)
    }

    fn input_values(&self) -> Box<dyn Iterator<Item = &InputValue<'a, T>> + '_> {
        match self {
            TypeDefinition::InputObject(ty) => Box::new(ty.fields.iter()),
            _ => Box::new(empty()),
        }
    }

    fn input_value(&self, name: &T::Value) -> Option<&InputValue<'a, T>> {
        self.input_values().find(|value| &value.name == name)
    }

    fn is_composite(&self) -> bool {
        match self {
            TypeDefinition::InputObject(_)
            | TypeDefinition::Interface(_)
            | TypeDefinition::Object(_)
            | TypeDefinition::Union(_) => true,
            TypeDefinition::Enum(_) | TypeDefinition::Scalar(_) => false,
        }
    }

    fn implements_interfaces(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            TypeDefinition::Enum(_)
            | TypeDefinition::InputObject(_)
            | TypeDefinition::Scalar(_)
            | TypeDefinition::Union(_) => Box::new(empty()),
            TypeDefinition::Object(definition) => {
                Box::new(definition.implements_interfaces.iter().map(AsRef::as_ref))
            }
            TypeDefinition::Interface(definition) => Box::new(
                definition
                    .implements_interfaces
                    .iter()
                    .map(AsRef::as_ref)
                    .chain(once(definition.name.as_ref())),
            ),
        }
    }

    fn implements_interface(&self, name: &str) -> bool {
        self.implements_interfaces()
            .find(|&interface| interface == name)
            .is_some()
    }
}
