use graphql_parser::schema::Document;
use kono_aspect::{AspectExt, Error, ObjectValue};
use kono_executor::{join, Resolver};

use super::{Field, Schema, Type};

pub fn introspection<C>(
    schema: Document<'static, String>,
) -> impl Resolver<Context = C, Error = Error, Value = ObjectValue>
where
    C: 'static,
{
    join(
        join(Schema::with_env(schema), Type::resolver()),
        Field::resolver(),
    )
}
