use std::marker::PhantomData;

use graphql_parser::schema;
use kono_macros::kono;

use super::Schema;

pub struct Introspection<C = ()> {
    _context: PhantomData<C>,
}

#[kono]
impl<C> Aspect for Introspection<C>
where
    C: 'static,
{
    type Context = C;
    type Environment = schema::Document<'static, String>;

    #[kono::query]
    fn schema(environment: &schema::Document<'static, String>) -> Schema<C> {
        Schema::new(environment.to_owned())
    }
}
