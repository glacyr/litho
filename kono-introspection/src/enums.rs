use std::marker::PhantomData;

use graphql_parser::schema;
use kono_macros::kono;

use super::kono;

pub struct EnumValue<C> {
    _context: PhantomData<C>,
    value: schema::EnumValue<'static, String>,
}

impl<C> EnumValue<C> {
    pub fn new(value: &schema::EnumValue<'static, String>) -> EnumValue<C> {
        EnumValue {
            _context: PhantomData,
            value: value.to_owned(),
        }
    }
}

#[kono(rename = "__EnumValue")]
impl<C> EnumValue<C>
where
    C: 'static,
{
    type Context = C;

    pub fn name(&self) -> &str {
        &self.value.name
    }

    pub fn description(&self) -> Option<&str> {
        self.value.description.as_deref()
    }

    pub fn is_deprecated(&self) -> bool {
        self.value
            .directives
            .iter()
            .find(|directive| directive.name == "deprecated")
            .is_some()
    }

    pub fn deprecation_reason(&self) -> Option<&str> {
        None
    }
}
