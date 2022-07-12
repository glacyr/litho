use graphql_parser::schema::Document;
use kono_aspect::{AspectExt, Error, ObjectValue};
use kono_executor::{join, Resolver};

mod field;
mod introspection;
mod schema;
mod ty;

use field::Field;
use introspection::Introspection;
use schema::{Schema, SchemaExt};
use ty::{Type, TypeDefinitionExt};

pub fn introspection<C>(
    schema: Document<'static, String>,
) -> impl Resolver<Context = C, Error = Error, Value = ObjectValue>
where
    C: 'static,
{
    join(
        join(
            join(Introspection::with_env(schema), Schema::resolver()),
            Type::resolver(),
        ),
        Field::resolver(),
    )
}
