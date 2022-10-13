use nom::error::{ErrorKind, ParseError};
use nom::{Err, IResult, Parser};

use crate::RecoverableParser;

pub trait Recognizer<I, E> {
    fn recognize(&self, input: I) -> IResult<I, (), E>;

    fn or<R>(self, other: R) -> Or<Self, R>
    where
        Self: Sized,
    {
        Or(self, other)
    }

    fn by_ref(&self) -> ByRef<Self>
    where
        Self: Sized,
    {
        ByRef(self)
    }
}

impl<I, E, R> Recognizer<I, E> for Box<R>
where
    R: Recognizer<I, E> + ?Sized,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.as_ref().recognize(input)
    }
}

impl<I, E, R> Recognizer<I, E> for &R
where
    R: Recognizer<I, E> + ?Sized,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        (*self).recognize(input)
    }
}

impl<I, E, R> Recognizer<I, E> for &mut R
where
    R: Recognizer<I, E> + ?Sized,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        (**self).recognize(input)
    }
}

pub struct ByRef<'a, R>(&'a R);

impl<R, I, E> Recognizer<I, E> for ByRef<'_, R>
where
    R: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        (self.0).recognize(input)
    }
}

struct Terminal<F>(F);

impl<I, O, E, F> Parser<I, O, E> for Terminal<F>
where
    F: FnMut(I) -> IResult<I, O, E>,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        (self.0)(input)
    }
}

impl<I, O, E, F> RecoverableParser<I, O, E> for Terminal<F>
where
    I: Clone,
    F: Fn(I) -> IResult<I, O, E>,
{
    fn parse<R>(&self, input: I, _recovery_point: R) -> IResult<I, O, E> {
        (self.0)(input)
    }
}

impl<I, E, F, O> Recognizer<I, E> for Terminal<F>
where
    I: Clone,
    F: Fn(I) -> IResult<I, O, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        let clone = input.clone();
        (self.0)(input).map(|_| (clone, ()))
    }
}

pub fn terminal<F, I, O, E>(
    parser: F,
) -> impl Parser<I, O, E> + Recognizer<I, E> + RecoverableParser<I, O, E>
where
    I: Clone,
    F: Fn(I) -> IResult<I, O, E>,
{
    Terminal(parser)
}

pub struct Fail;

impl<I, E> Recognizer<I, E> for Fail
where
    E: ParseError<I>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        Err(Err::Error(E::from_error_kind(input, ErrorKind::Fail)))
    }
}

pub struct Or<A, B>(A, B);

impl<I, E, A, B> Recognizer<I, E> for Or<A, B>
where
    I: Clone,
    A: Recognizer<I, E>,
    B: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0
            .recognize(input.clone())
            .or_else(move |_| self.1.recognize(input))
    }
}
