use nom::error::ParseError;
use nom::Parser;

use super::{Input, Recognizer, RecoverableParser};

/// Recoverable parser that always succeeds, with `Some(_)` if the given
/// underlying parser succeeds and with `None` if the underlying parser fails.
pub struct Opt<P>(P);

impl<I, E, P> Recognizer<I, E> for Opt<P>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, O, E, P> RecoverableParser<I, Option<O>, E> for Opt<P>
where
    I: Clone + Input,
    E: ParseError<I>,
    P: RecoverableParser<I, O, E>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, Option<O>, E>
    where
        R: Recognizer<I, E>,
    {
        nom::combinator::opt(self.0.parser(recovery_point))
    }
}

/// Returns a recoverable parser that always succeeds, with `Some(_)` if the
/// given `parser` succeeds and with `None` if the given `parser` fails.
pub fn opt<P>(parser: P) -> Opt<P> {
    Opt(parser)
}
