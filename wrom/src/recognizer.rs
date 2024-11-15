use nom::error::ParseError;
use nom::IResult;

use crate::RecoverableParser;

/// Implemented by types (usually parsers) that can recognize something from an
/// input, without consuming it. This is the starting point of your recoverable
/// parser.
pub trait Recognizer<I, E> {
    /// Returns a parser that should return `Ok(_)` when it recognizes the start
    /// of a parsing rule and `Err(_)` otherwise, without consuming its input.
    fn recognize(&self, input: I) -> IResult<I, (), E>;

    /// Returns a recognizer that succeeds when either `self` or the `other`
    /// recognizer succeeds.
    fn or<R>(self, other: R) -> impl Recognizer<I, E>
    where
        I: Clone,
        E: ParseError<I>,
        R: Recognizer<I, E>,
        Self: Sized,
    {
        Or(self, other)
    }

    fn non_recursive(&self) -> impl Recognizer<I, E> {
        self
    }
}

impl<I, E, R> Recognizer<I, E> for &R
where
    R: Recognizer<I, E> + ?Sized,
{
    #[inline(always)]
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        (*self).recognize(input)
    }

    #[inline(always)]
    fn non_recursive(&self) -> impl Recognizer<I, E> {
        <R as Recognizer<_, _>>::non_recursive(&self)
    }
}

#[derive(Clone)]
struct Terminal<F>(F);

impl<I, O, E, F> RecoverableParser<I, O, E> for Terminal<F>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E> + Clone,
{
    #[inline(always)]
    fn parse<R>(&self, input: I, _recovery_point: R) -> IResult<I, O, E> {
        self.0(input)
    }
}

impl<I, E, F, O> Recognizer<I, E> for Terminal<F>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E> + Clone,
{
    #[inline(always)]
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        (self.0)(input).map(|(input, _)| (input, ()))
    }
}

/// Returns a recoverable parser based on a parser function that will serve both
/// as its parser and as its recognizer.
pub fn terminal<F, I, O, E>(parser: F) -> impl RecoverableParser<I, O, E>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E> + Clone,
{
    Terminal(parser)
}

pub struct Or<A, B>(A, B);

impl<I, E, A, B> Recognizer<I, E> for Or<A, B>
where
    I: Clone,
    E: ParseError<I>,
    A: Recognizer<I, E>,
    B: Recognizer<I, E>,
{
    #[inline(always)]
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        if let Ok(result) = self.0.recognize(input.clone()) {
            return Ok(result);
        }

        self.1.recognize(input)
    }
}
