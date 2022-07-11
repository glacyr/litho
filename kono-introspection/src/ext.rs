use graphql_parser::schema::{Text, TypeDefinition};

pub trait TypeDefinitionExt<'a, T>
where
    T: Text<'a>,
{
    fn name(&self) -> &T::Value;
    fn description(&self) -> Option<&str>;
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

    fn description(&self) -> Option<&str> {
        match self {
            TypeDefinition::Scalar(ty) => ty.description.as_deref(),
            TypeDefinition::Object(ty) => ty.description.as_deref(),
            TypeDefinition::Interface(ty) => ty.description.as_deref(),
            TypeDefinition::Union(ty) => ty.description.as_deref(),
            TypeDefinition::Enum(ty) => ty.description.as_deref(),
            TypeDefinition::InputObject(ty) => ty.description.as_deref(),
        }
    }
}
