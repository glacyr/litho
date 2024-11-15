use nom::error::ParseError;
use nom::IResult;

use crate::RecoverableParser;

/// Implemented by types (usually parsers) that can recognize something from an
/// input, without consuming it. This is the starting point of your recoverable
/// parser.
pub trait Recognizer<I, E>: Default + Copy {
    /// Returns a parser that should return `Ok(_)` when it recognizes the start
    /// of a parsing rule and `Err(_)` otherwise, without consuming its input.
    fn recognize(self, input: I) -> IResult<I, (), E>;

    /// Returns a recognizer that succeeds when either `self` or the `other`
    /// recognizer succeeds.
    fn or(self, other: Self) -> Self;
}

#[derive(Clone)]
struct Terminal<R, F>(R, F);

impl<I, O, E, R, F> RecoverableParser<I, O, E, R> for Terminal<R, F>
where
    I: Clone,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    F: Fn(I) -> IResult<I, O, E> + Clone,
{
    fn recovery_point(&self) -> R {
        self.0
    }

    #[inline(always)]
    fn parse(&self, input: I, _recovery_point: R) -> IResult<I, O, E> {
        self.1(input)
    }
}

/// Returns a recoverable parser based on a parser function that will serve both
/// as its parser and as its recognizer.
pub fn terminal<R, F, I, O, E>(recovery_point: R, parser: F) -> impl RecoverableParser<I, O, E, R>
where
    I: Clone,
    E: ParseError<I>,
    R: Recognizer<I, E>,
    F: Fn(I) -> IResult<I, O, E> + Clone,
{
    Terminal(recovery_point, parser)
}
