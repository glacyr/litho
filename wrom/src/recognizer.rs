use nom::error::ParseError;
use nom::{IResult, Parser};

use crate::RecoverableParser;

/// Implemented by types (usually parsers) that can recognize something from an
/// input, without consuming it. This is the starting point of your recoverable
/// parser.
pub trait Recognizer<I, E> {
    /// Returns a parser that should return `Ok(_)` when it recognizes the start
    /// of a parsing rule and `Err(_)` otherwise, without consuming its input.
    fn recognizer(&self) -> impl Parser<I, (), E>;

    /// Returns a recognizer that succeeds when either `self` or the `other`
    /// recognizer succeeds.
    fn or<R>(self, other: R) -> Or<Self, R>
    where
        Self: Sized,
    {
        Or(self, other)
    }
}

impl<I, E, R> Recognizer<I, E> for &R
where
    R: Recognizer<I, E> + ?Sized,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        (*self).recognizer()
    }
}

struct Terminal<F>(F);

impl<I, O, E, F> RecoverableParser<I, O, E> for Terminal<F>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
    fn parser<R>(&self, _recovery_point: R) -> impl Parser<I, O, E> {
        &self.0
    }
}

impl<I, E, F, O> Recognizer<I, E> for Terminal<F>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        nom::combinator::peek(&self.0).map(|_| ())
    }
}

/// Returns a recoverable parser based on a parser function that will serve both
/// as its parser and as its recognizer.
pub fn terminal<F, I, O, E>(parser: F) -> impl RecoverableParser<I, O, E>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
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
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer().or(self.1.recognizer())
    }
}
