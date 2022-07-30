use std::borrow::Cow;

/// Should be implemented by any type that has a name. Used for distinguishing
/// union values, resolving fragments, etc.
pub trait Typename {
    /// Should return the name of this type.
    fn typename(&self) -> Cow<str>;
}
