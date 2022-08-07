use graphql_parser::schema;

use super::Emit;

/// Reference to a GraphQL type.
pub enum Type {
    /// Reference to a named object type (i.e. an [`ItemType`](super::ItemType)).
    Named(String),

    /// Reference to a scalar type (i.e. an [`ItemScalar`](super::ItemScalar))
    /// or a built-in scalar (e.g. `String`, `Int`, `Boolean`).
    Scalar(String),

    /// Reference to an optional type. Note that this will still reference to
    /// the actual type.
    Optional(Box<Type>),

    /// Reference to a list type. Note that this will still reference to the
    /// element type.
    List(Box<Type>),
}

impl Emit for Type {
    type Target = schema::Type<'static, String>;

    fn emit(self) -> schema::Type<'static, String> {
        fn into_inner(ty: Type) -> schema::Type<'static, String> {
            match ty {
                Type::Named(name) => schema::Type::NamedType(name),
                Type::Scalar(name) => schema::Type::NamedType(name),
                Type::List(list) => schema::Type::ListType(Box::new((*list).emit())),
                Type::Optional(optional) => into_inner(*optional),
            }
        }

        match self {
            Type::Optional(ty) => into_inner(*ty),
            _ => schema::Type::NonNullType(Box::new(into_inner(self))),
        }
    }
}
