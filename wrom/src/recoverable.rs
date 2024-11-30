/// Analogous to `Result<T, E>`, represents a token that is either present
/// (successfully parsed), or interpolated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recoverable<T, E> {
    /// Represents a value that is present (i.e. a successful parse).
    Present(T),

    /// Represents a value that is missed, by its replacement (i.e. after a
    /// recovered unsuccessful parse).
    Missing(E),
}

impl<T, E> Recoverable<T, E> {
    /// Returns an [`Option`] that is [`Option::Some`] when `T` was parsed
    /// successfully, or [`Option::None`] otherwise.
    #[inline(always)]
    pub fn ok(&self) -> Option<&T> {
        match self {
            Recoverable::Present(value) => Some(value),
            Recoverable::Missing(_) => None,
        }
    }
}

impl<T, E> From<T> for Recoverable<T, E> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Recoverable::Present(value)
    }
}

mod mock {
    use arbitrary::{Arbitrary, Result, Unstructured};

    use super::Recoverable;

    impl<'a, T, E> Arbitrary<'a> for Recoverable<T, E>
    where
        T: Arbitrary<'a>,
    {
        fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
            Ok(Recoverable::Present(u.arbitrary()?))
        }
    }
}

mod display {
    use std::fmt::{Display, Formatter, Result};

    use super::Recoverable;

    impl<T, E> Display for Recoverable<T, E>
    where
        T: Display,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Recoverable::Present(value) => value.fmt(f),
                Recoverable::Missing(_) => f.write_str("(missing)"),
            }
        }
    }
}
