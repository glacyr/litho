use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Error {
    message: String,
}

impl kono_executor::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error {
            message: format!("{}", msg),
        }
    }
}
