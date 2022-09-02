use std::iter::empty;

use graphql_parser::schema::{Definition, Document, Field, Text, Type, TypeDefinition};

pub trait DocumentExt<'a, T>
where
    T: Text<'a>,
{
    fn type_definitions(&self) -> Box<dyn Iterator<Item = &TypeDefinition<'a, T>> + '_>;

    fn type_definition<S>(&self, name: S) -> Option<&TypeDefinition<'a, T>>
    where
        S: AsRef<str>;
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

    fn is_composite(&self) -> bool;
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

    fn is_composite(&self) -> bool {
        match self {
            TypeDefinition::InputObject(_)
            | TypeDefinition::Interface(_)
            | TypeDefinition::Object(_)
            | TypeDefinition::Union(_) => true,
            TypeDefinition::Enum(_) | TypeDefinition::Scalar(_) => false,
        }
    }
}
