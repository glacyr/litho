use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use super::{Error, Intermediate, Value};

pub trait Resolver {
    type Context;
    type Error: Error;
    type Value;

    fn typename(&self, object_value: &Self::Value, context: &Self::Context) -> Option<String>;

    fn can_resolve<'a>(
        &self,
        object_ty: (),
        object_value: &Self::Value,
        field_name: &str,
        context: &Self::Context,
    ) -> bool;

    fn resolve<'a>(
        &'a self,
        object_ty: (),
        object_value: &'a Self::Value,
        field_name: &'a str,
        argument_values: &'a HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<Self::Value>, Self::Error>> + 'a>>;
}
