use std::any::type_name;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use kono_executor::{Error, Intermediate, Value};

use super::ObjectValue;

pub trait ResolveField {
    type Context: 'static;
    type Error: Error;

    fn typename(&self) -> &str;

    fn can_resolve_field(&self, _field: &str) -> bool {
        false
    }

    fn resolve_field<'a>(
        &'a self,
        _field: &'a str,
        _args: &'a HashMap<String, Value>,
        _context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<ObjectValue>, Self::Error>> + 'a>> {
        todo!()
    }
}
