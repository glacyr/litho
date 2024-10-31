use nom::error::{ErrorKind, ParseError};
use nom::{Err, Parser};

use super::{Recognizer, RecoverableParser};

pub struct Recursive<P> {
    max_depth: usize,
    parser_fn: fn(max_depth: usize) -> P,
}

/// Returns a recursive wrapper that breaks infinite recursion and prevents
/// stack overflows.
///
/// The returned recursive parser serves two purposes: first it lazily calls the
/// given `parser_fn` instead of calling it immediately (breaking infinite
/// recursion), and it keeps track of the recursion depth and returns an error
/// when `max_depth` is reached, thereby preventing a stack overflow.
///
/// ```rust
/// # use wrom::mock::char;
/// # use wrom::{alt, recursive, Input, RecoverableParser};
/// #
/// fn square_brackets<'a, I>(max_depth: usize) -> impl RecoverableParser<I, usize, ()> + 'a
/// where
///     I: Input<Item = char> + 'a,
/// {
///     char('[')
///         .and(alt((
///             char(']').map(|_| 0),
///             recursive(max_depth, square_brackets),
///         )))
///         .map(|(_, i)| i + 1)
///         .boxed()
/// }
///
/// assert_eq!(square_brackets(128).parse("[  [ -[]]-]").unwrap(), 3);
/// assert!(square_brackets(1).parse("[[[]]]").is_err());
/// ```
pub fn recursive<P, I, O, E>(
    max_depth: usize,
    parser_fn: fn(max_depth: usize) -> P,
) -> impl RecoverableParser<I, O, E>
where
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    Recursive {
        max_depth,
        parser_fn,
    }
}

impl<I, E, R> Recognizer<I, E> for Recursive<R>
where
    E: ParseError<I>,
    R: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        |input| match self.max_depth {
            0 => Err(Err::Failure(E::from_error_kind(input, ErrorKind::Fail))),
            n => (self.parser_fn)(n - 1).recognizer().parse(input),
        }
    }
}

impl<I, O, E, P> RecoverableParser<I, O, E> for Recursive<P>
where
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, O, E>
    where
        R: Recognizer<I, E>,
    {
        move |input| match self.max_depth {
            0 => Err(Err::Failure(E::from_error_kind(input, ErrorKind::Fail))),
            n => (self.parser_fn)(n - 1).parser(&recovery_point).parse(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::char;
    use crate::{alt, recursive, Input, RecoverableParser};

    #[test]
    fn test_recursion() {
        fn square_brackets<'a, I>(max_depth: usize) -> impl RecoverableParser<I, usize, ()> + 'a
        where
            I: Input<Item = char> + 'a,
        {
            char('[')
                .and(alt((
                    char(']').map(|_| 0),
                    recursive(max_depth, square_brackets),
                )))
                .map(|(_, i)| i + 1)
                .boxed()
        }

        assert_eq!(square_brackets(128).parse("[  [ -[]]-]").unwrap(), 3);
        assert_eq!(square_brackets(2).parse("[  [ -[]]-]").unwrap(), 3);
        assert!(square_brackets(1).parse("[  [ -[]]-]").is_err());
        assert!(square_brackets(0).parse("[[]]").is_err());
    }
}
