use graphql_parser::schema::Document;

use kono_aspect::{AspectExt, Error, ObjectValue};
use kono_executor::Resolver;

use super::{Field, Schema, Type};

pub fn introspection<C>(
    schema: Document<'static, String>,
) -> impl Resolver<Context = C, Error = Error, Value = ObjectValue>
where
    C: 'static,
{
    (
        Schema::with_env(schema),
        Type::resolver(),
        Field::resolver(),
    )
}
