use std::marker::PhantomData;
use std::rc::Rc;

use graphql_parser::schema;
use kono_macros::{kono, Kono};

use super::{Field, InputValue, SchemaExt};

#[derive(Kono)]
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

    pub fn by_name(schema: &Rc<schema::Document<'static, String>>, name: &str) -> Type<C> {
        let inner = match schema.type_definition(name) {
            Some(definition) => InnerType::Definition(definition.to_owned()),
            None => InnerType::Unknown(name.to_owned()),
        };

        Type {
            _context: PhantomData,
            schema: schema.to_owned(),
            inner,
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

    fn kind(&self) -> TypeKind {
        match &self.inner {
            InnerType::Definition(definition) => match definition {
                schema::TypeDefinition::Enum(_) => TypeKind::Enum,
                schema::TypeDefinition::InputObject(_) => TypeKind::InputObject,
                schema::TypeDefinition::Interface(_) => TypeKind::Interface,
                schema::TypeDefinition::Object(_) => TypeKind::Object,
                schema::TypeDefinition::Scalar(_) => TypeKind::Scalar,
                schema::TypeDefinition::Union(_) => TypeKind::Union,
            },
            InnerType::List(_) => TypeKind::List,
            InnerType::NonNull(_) => TypeKind::NonNull,
            InnerType::Unknown(_) => TypeKind::Scalar,
        }
    }

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

    fn fields(&self) -> Option<Vec<Field<C>>> {
        let fields = match &self.inner {
            InnerType::Definition(schema::TypeDefinition::Object(object)) => &object.fields,
            InnerType::Definition(schema::TypeDefinition::Interface(interface)) => {
                &interface.fields
            }
            _ => return None,
        };

        Some(
            fields
                .iter()
                .map(|field| Field::new(&self.schema, field))
                .collect(),
        )
    }

    fn interfaces(&self) -> Option<Vec<Type<C>>> {
        match &self.inner {
            InnerType::Definition(schema::TypeDefinition::Object(object)) => Some(
                object
                    .implements_interfaces
                    .iter()
                    .map(|name| Type::by_name(&self.schema, name))
                    .collect(),
            ),
            _ => None,
        }
    }

    fn possible_types(&self) -> Option<Vec<Type<C>>> {
        match &self.inner {
            InnerType::Definition(schema::TypeDefinition::Interface(interface)) => Some(
                self.schema
                    .definitions
                    .iter()
                    .filter_map(|definition| match definition {
                        schema::Definition::TypeDefinition(ty) => match ty {
                            schema::TypeDefinition::Object(object)
                                if object.implements_interfaces.contains(&interface.name) =>
                            {
                                Some(ty)
                            }
                            _ => None,
                        },
                        _ => None,
                    })
                    .map(|definition| Type::new(&self.schema, definition))
                    .collect(),
            ),
            _ => None,
        }
    }

    fn enum_values(&self) -> Option<Vec<()>> {
        None
    }

    fn input_fields(&self) -> Option<Vec<InputValue<C>>> {
        None
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
