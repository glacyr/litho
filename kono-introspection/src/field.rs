use std::marker::PhantomData;
use std::rc::Rc;

use graphql_parser::schema;
use kono_macros::kono;

use super::Type;

pub struct Field<C = ()> {
    _context: PhantomData<C>,
    schema: Rc<schema::Document<'static, String>>,
    field: schema::Field<'static, String>,
}

impl<C> Field<C> {
    pub fn new(
        schema: &Rc<schema::Document<'static, String>>,
        field: &schema::Field<'static, String>,
    ) -> Field<C> {
        Field {
            _context: PhantomData,
            schema: schema.to_owned(),
            field: field.to_owned(),
        }
    }
}

#[kono]
impl<C> Aspect for Field<C>
where
    C: 'static,
{
    type Context = C;

    #[kono::field]
    fn name(&self) -> &str {
        &self.field.name
    }

    #[kono::field]
    fn description(&self) -> Option<&str> {
        self.field.description.as_deref()
    }

    #[kono::field]
    fn r#type(&self) -> Type<C> {
        Type::ty(&self.schema, &self.field.field_type)
    }

    #[kono::field]
    fn is_deprecated(&self) -> bool {
        false
    }

    #[kono::field]
    fn deprecation_reason(&self) -> Option<&str> {
        None
    }
}
