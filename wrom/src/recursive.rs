use nom::{IResult, Parser};

use super::{Recognizer, RecoverableParser};

pub struct Recursive<'a, I, O, E>(Box<dyn ErasedRecoverableParser<I, O, E> + 'a>);

pub struct RecursiveRecognizer<'a, R>(&'a R)
where
    R: ?Sized;

impl<R, I, E> Recognizer<I, E> for RecursiveRecognizer<'_, R>
where
    R: ErasedRecognizer<I, E> + ?Sized,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        |input| self.0.recognize(input)
    }
}

trait ErasedRecognizer<I, E> {
    fn recognize(&self, input: I) -> IResult<I, (), E>;
}

trait ErasedRecoverableParser<I, O, E> {
    fn recognize(&self, input: I) -> IResult<I, (), E>;
    fn parse(&self, input: I, recovery_point: &dyn ErasedRecognizer<I, E>) -> IResult<I, O, E>;
}

impl<I, E, R> ErasedRecognizer<I, E> for R
where
    I: Iterator,
    R: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        Recognizer::recognizer(&self).parse(input)
    }
}

impl<I, O, E, P> ErasedRecoverableParser<I, O, E> for fn() -> P
where
    I: Iterator,
    P: RecoverableParser<I, O, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        Recognizer::recognizer(&self()).parse(input)
    }

    fn parse(&self, input: I, recovery_point: &dyn ErasedRecognizer<I, E>) -> IResult<I, O, E> {
        RecoverableParser::parser(&self(), RecursiveRecognizer(recovery_point)).parse(input)
    }
}

pub fn recursive<'a, P, I, O, E>(parser: fn() -> P) -> Recursive<'a, I, O, E>
where
    I: Iterator,
    P: RecoverableParser<I, O, E> + 'a,
{
    Recursive(Box::new(parser))
}

impl<I, O, E> Recognizer<I, E> for Recursive<'_, I, O, E>
where
    I: Iterator,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        |input| self.0.recognize(input)
    }
}

impl<I, O, E> RecoverableParser<I, O, E> for Recursive<'_, I, O, E>
where
    I: Iterator,
{
    fn parser<R2>(&self, recovery_point: R2) -> impl Parser<I, O, E>
    where
        R2: Recognizer<I, E>,
    {
        move |input| self.0.parse(input, &recovery_point)
    }
}
