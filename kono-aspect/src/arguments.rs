use std::collections::HashMap;

use kono_schema::InputValue;

use kono_executor::{Error, Value};

pub trait ArgumentType<Env> {
    fn schema(environment: &Env) -> Vec<InputValue>;

    fn from_args<E>(args: &HashMap<String, Value>, environment: &Env) -> Result<Self, E>
    where
        Self: Sized,
        E: Error;
}
