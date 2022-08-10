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

    /// Called when a request's document contains no operations.
    fn missing_operation() -> Self {
        Self::custom(format_args!("document does not contain any operations"))
    }

    /// Called when an operation defines a variable for which no value is
    /// provided.
    fn missing_variable_value(name: &str) -> Self {
        Self::custom(format_args!(
            "document is missing value for variable `{}`",
            name
        ))
    }

    /// Called when a document contains a float literal that cannot be expressed
    /// in json.
    fn incoercible_float_literal(value: f64) -> Self {
        Self::custom(format_args!(
            "float literal `{}` cannot be coerced to json",
            value
        ))
    }

    /// Called when a document contains an int literal that cannot be expressed
    /// in json.
    fn incoercible_int_value(value: &str) -> Self {
        Self::custom(format_args!(
            "int literal `{}` cannot be coerced to json",
            value
        ))
    }

    /// Called when a document defines a required variable for which no value is
    /// provided.
    fn missing_value() -> Self {
        Self::custom(format_args!("missing value"))
    }

    /// Called when a document is given a value for a variable with a type that
    /// doesn't match.
    fn unexpected_value_type(expected: &str) -> Self {
        Self::custom(format_args!(
            "value has unexpected type, expected `{}`",
            expected
        ))
    }
}
