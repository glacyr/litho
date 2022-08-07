mod aspect;
mod record;
mod reference;
mod resolver;
mod traits;
mod types;
mod value;

pub use aspect::Aspect;
pub use record::{Record, RecordResolver};
pub use reference::Reference;
pub use resolver::{AspectExt, AspectResolver};
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
