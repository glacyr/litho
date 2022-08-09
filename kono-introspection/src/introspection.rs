use graphql_parser::schema::Document;

use kono_aspect::{AspectExt, Error, ObjectValue};
use kono_executor::Resolver;

use super::{EnumValue, Field, Schema, Type};

pub fn introspection<C>(
    schema: Document<'static, String>,
) -> impl Resolver<Context = C, Error = Error, Value = ObjectValue> + kono_schema::Schema
where
    C: 'static,
{
    (
        Schema::with_env(schema),
        EnumValue::resolver(),
        Field::resolver(),
        Type::resolver(),
    )
}
