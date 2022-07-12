use std::fmt::Display;

pub trait Error: Sized {
    fn custom<T>(msg: T) -> Self
    where
        T: Display;

    fn unknown_operation(name: &str, expected: &[&str]) -> Self {
        Self::custom(format_args!(
            "unknown operation `{}`, expected {:?}",
            name, expected
        ))
    }

    fn unspecified_operation(expected: &[&str]) -> Self {
        Self::custom(format_args!("document contains multiple operations, requires operation name to be specified, expected {:?}", expected))
    }
}
