use nom::error::ParseError;
use nom::IResult;

use super::{Input, Recognizer, RecoverableParser};

/// Recoverable parser that always succeeds, with `Some(_)` if the given
/// underlying parser succeeds and with `None` if the underlying parser fails.
pub struct Opt<P>(P);

impl<I, E, P> Recognizer<I, E> for Opt<P>
where
    P: Recognizer<I, E>,
{
    #[inline(always)]
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O, E, P> RecoverableParser<I, Option<O>, E> for Opt<P>
where
    I: Clone + Input,
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    #[inline(always)]
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, Option<O>, E>
    where
        R: Recognizer<I, E>,
    {
        match self.0.parse(input.clone(), recovery_point) {
            Ok((input, value)) => Ok((input, Some(value))),
            Err(_) => Ok((input, None)),
        }
    }
}

/// Returns a recoverable parser that always succeeds, with `Some(_)` if the
/// given `parser` succeeds and with `None` if the given `parser` fails.
pub fn opt<P>(parser: P) -> Opt<P> {
    Opt(parser)
}
