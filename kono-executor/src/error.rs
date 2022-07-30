use std::fmt::Display;

/// Should be implemented by error types.
pub trait Error: Sized {
    /// Should return a new error with the given message. This is the only
    /// method that needs to be implemented. All other methods can call this
    /// method indirectly.
    fn custom<T>(msg: T) -> Self
    where
        T: Display;

    /// Called when [`Resolver::can_resolve`](super::Resolver::can_resolve)
    /// returns `false`.
    fn unknown_field(ty: &str, name: &str) -> Self {
        Self::custom(format_args!("unknown field `{}` for type `{}`", name, ty))
    }

    /// Called when a request refers to an operation that is not present in its
    /// document.
    fn unknown_operation(name: &str, expected: &[&str]) -> Self {
        Self::custom(format_args!(
            "unknown operation `{}`, expected {:?}",
            name, expected
        ))
    }

    /// Called when a request's document contains multiple operations and its
    /// name isn't explicitly specified.
    fn unspecified_operation(expected: &[&str]) -> Self {
        Self::custom(format_args!("document contains multiple operations, requires operation name to be specified, expected {:?}", expected))
    }
}
