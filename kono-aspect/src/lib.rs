mod aspect;
mod mutation;
mod query;
mod reference;
mod resolve_field;
mod traits;
mod types;
mod value;

pub use aspect::{Aspect, AspectExt};
pub use mutation::Mutation;
pub use query::Query;
pub use reference::Reference;
pub use resolve_field::ResolveField;
pub use traits::IntoIntermediate;
pub use types::{InputType, OutputType};
pub use value::ObjectValue;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Error {
    message: String,
}

impl kono_executor::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error {
            message: format!("{}", msg),
        }
    }
}
