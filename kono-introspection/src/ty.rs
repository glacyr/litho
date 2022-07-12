use std::marker::PhantomData;
use std::rc::Rc;

use graphql_parser::schema;
use kono_macros::kono;

use super::{Field, SchemaExt};

pub struct Type<C = ()> {
    _context: PhantomData<C>,
    schema: Rc<schema::Document<'static, String>>,
    inner: InnerType,
}

#[derive(Clone)]
pub enum InnerType {
    Definition(schema::TypeDefinition<'static, String>),
    List(Box<InnerType>),
    NonNull(Box<InnerType>),
    Unknown(String),
}

impl<C> Type<C> {
    pub fn new(
        schema: &Rc<schema::Document<'static, String>>,
        definition: &schema::TypeDefinition<'static, String>,
    ) -> Type<C> {
        Type {
            _context: PhantomData,
            schema: schema.to_owned(),
            inner: InnerType::Definition(definition.to_owned()),
        }
    }

    pub fn ty(
        schema: &Rc<schema::Document<'static, String>>,
        ty: &schema::Type<'static, String>,
    ) -> Type<C> {
        fn map_ty(
            schema: &schema::Document<'static, String>,
            ty: &schema::Type<'static, String>,
        ) -> InnerType {
            match ty {
                schema::Type::ListType(ty) => InnerType::List(Box::new(map_ty(schema, ty))),
                schema::Type::NonNullType(ty) => InnerType::NonNull(Box::new(map_ty(schema, ty))),
                schema::Type::NamedType(name) => match schema.type_definition(name) {
                    Some(definition) => InnerType::Definition(definition.to_owned()),
                    None => InnerType::Unknown(name.to_owned()),
                },
            }
        }

        Type {
            _context: PhantomData,
            schema: schema.to_owned(),
            inner: map_ty(schema, ty),
        }
    }
}

#[kono]
impl<C> Aspect for Type<C>
where
    C: 'static,
{
    type Context = C;

    fn name(&self) -> Option<&str> {
        match &self.inner {
            InnerType::Definition(definition) => Some(definition.name()),
            InnerType::Unknown(unknown) => Some(unknown),
            _ => None,
        }
    }

    fn description(&self) -> Option<&str> {
        match &self.inner {
            InnerType::Definition(definition) => definition.description(),
            _ => None,
        }
    }

    fn fields(&self) -> Vec<Field<C>> {
        let fields = match &self.inner {
            InnerType::Definition(schema::TypeDefinition::Object(object)) => &object.fields,
            _ => return vec![],
        };

        fields
            .iter()
            .map(|field| Field::new(&self.schema, field))
            .collect()
    }

    fn of_type(&self) -> Option<Type<C>> {
        match &self.inner {
            InnerType::List(inner) | InnerType::NonNull(inner) => Some(Type {
                _context: PhantomData,
                schema: self.schema.to_owned(),
                inner: *inner.to_owned(),
            }),
            _ => None,
        }
    }
}

pub trait TypeDefinitionExt<'a, T>
where
    T: schema::Text<'a>,
{
    fn name(&self) -> &T::Value;
    fn description(&self) -> Option<&str>;
}

impl<'a, T> TypeDefinitionExt<'a, T> for schema::TypeDefinition<'a, T>
where
    T: schema::Text<'a>,
{
    fn name(&self) -> &T::Value {
        match self {
            schema::TypeDefinition::Scalar(ty) => &ty.name,
            schema::TypeDefinition::Object(ty) => &ty.name,
            schema::TypeDefinition::Interface(ty) => &ty.name,
            schema::TypeDefinition::Union(ty) => &ty.name,
            schema::TypeDefinition::Enum(ty) => &ty.name,
            schema::TypeDefinition::InputObject(ty) => &ty.name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            schema::TypeDefinition::Scalar(ty) => ty.description.as_deref(),
            schema::TypeDefinition::Object(ty) => ty.description.as_deref(),
            schema::TypeDefinition::Interface(ty) => ty.description.as_deref(),
            schema::TypeDefinition::Union(ty) => ty.description.as_deref(),
            schema::TypeDefinition::Enum(ty) => ty.description.as_deref(),
            schema::TypeDefinition::InputObject(ty) => ty.description.as_deref(),
        }
    }
}
