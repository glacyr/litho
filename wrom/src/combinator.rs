use nom::error::ParseError;
use nom::IResult;

use super::{Input, Recognizer, RecoverableParser};

/// Recoverable parser that always succeeds, with `Some(_)` if the given
/// underlying parser succeeds and with `None` if the underlying parser fails.
pub struct Opt<P>(P);

impl<I, O, E, R, P> RecoverableParser<I, Option<O>, E, R> for Opt<P>
where
    I: Clone + Input,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    P: RecoverableParser<I, O, E, R>,
{
    fn recovery_point(&self) -> R {
        self.0.recovery_point()
    }

    fn parse(&self, input: I, recovery_point: R) -> IResult<I, Option<O>, E> {
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
