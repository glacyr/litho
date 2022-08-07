use std::marker::PhantomData;
use std::rc::Rc;

use graphql_parser::schema;

use kono_macros::kono;

use super::{kono, Type};

pub struct InputValue<C> {
    _context: PhantomData<C>,
    schema: Rc<schema::Document<'static, String>>,
    value: schema::InputValue<'static, String>,
}

impl<C> InputValue<C> {
    pub fn new(
        schema: &Rc<schema::Document<'static, String>>,
        value: &schema::InputValue<'static, String>,
    ) -> InputValue<C> {
        InputValue {
            _context: PhantomData,
            schema: schema.to_owned(),
            value: value.to_owned(),
        }
    }
}

#[kono]
impl<C> Aspect for InputValue<C>
where
    C: 'static,
{
    type Environment = schema::Document<'static, String>;

    fn name(&self) -> &str {
        &self.value.name
    }

    fn description(&self) -> Option<&str> {
        self.value.description.as_deref()
    }

    fn r#type(&self) -> Type<C> {
        Type::ty(&self.schema, &self.value.value_type)
    }
}
