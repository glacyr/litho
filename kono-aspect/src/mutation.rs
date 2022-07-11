use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use kono_executor::{Intermediate, Value};

use super::ObjectValue;

pub trait Mutation {
    type Context;
    type Error;

    fn can_mutate(field: &str) -> bool {
        false
    }

    fn mutate<'a>(
        field: &'a str,
        args: HashMap<String, Value>,
        context: &'a Self::Context,
    ) -> Pin<Box<dyn Future<Output = Result<Intermediate<ObjectValue>, Self::Error>>>> {
        todo!()
    }
}
