use super::Missing;

#[derive(Debug, Clone)]
pub enum Recoverable<T, M>
where
    M: Missing,
{
    Present(T),
    Missing(M::Error),
}

impl<T, M> Recoverable<T, M>
where
    M: Missing,
{
    pub fn ok(&self) -> Option<&T> {
        match self {
            Recoverable::Present(value) => Some(value),
            Recoverable::Missing(_) => None,
        }
    }
}

impl<T, M> From<T> for Recoverable<T, M>
where
    M: Missing,
{
    fn from(value: T) -> Self {
        Recoverable::Present(value)
    }
}

mod display {
    use std::fmt::{Display, Formatter, Result};

    use super::{Missing, Recoverable};

    impl<T, M> Display for Recoverable<T, M>
    where
        T: Display,
        M: Missing,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Recoverable::Present(value) => value.fmt(f),
                Recoverable::Missing(_) => f.write_str("(missing)"),
            }
        }
    }
}
