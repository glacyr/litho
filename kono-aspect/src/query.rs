use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use kono_executor::{Intermediate, Value};

use super::ObjectValue;

pub trait Query {
    type Context;
    type Error;
    type Environment;

    fn can_query(_environment: &Self::Environment, _field: &str, _context: &Self::Context) -> bool {
        true
    }

    fn query<'a>(
        _environment: &'a Self::Environment,
        _field: &'a str,
        _args: HashMap<String, Value>,
        _context: &'a Self::Context,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Intermediate<ObjectValue<Self::Context, Self::Error>>,
                        Self::Error,
                    >,
                > + 'a,
        >,
    > {
        todo!()
    }
}
