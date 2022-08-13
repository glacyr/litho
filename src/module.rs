use kono_aspect::{Error, ObjectValue};
use kono_executor::Resolver;
use kono_schema::Schema;

pub trait Module<C>: Resolver<Context = C, Error = Error, Value = ObjectValue> + Schema {}

impl<T> Module<T::Context> for T where T: Resolver<Error = Error, Value = ObjectValue> + Schema {}
