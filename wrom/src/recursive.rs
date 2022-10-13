use nom::IResult;

use super::{Recognizer, RecoverableParser};

pub struct Recursive<'a, I, O, E>(Box<dyn ErasedRecoverableParser<I, O, E> + 'a>);

trait ErasedRecoverableParser<I, O, E> {
    fn recognize(&self, input: I) -> IResult<I, (), E>;
    fn parse(&self, input: I, recovery_point: &dyn Recognizer<I, E>) -> IResult<I, O, E>;
}

impl<I, O, E, P> ErasedRecoverableParser<I, O, E> for fn() -> P
where
    I: Iterator,
    P: RecoverableParser<I, O, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        Recognizer::recognize(&self(), input)
    }

    fn parse(&self, input: I, recovery_point: &dyn Recognizer<I, E>) -> IResult<I, O, E> {
        RecoverableParser::parse(&self(), input, recovery_point)
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
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O, E> RecoverableParser<I, O, E> for Recursive<'_, I, O, E>
where
    I: Iterator,
{
    fn parse<R2>(&self, input: I, recovery_point: R2) -> IResult<I, O, E>
    where
        R2: Recognizer<I, E>,
    {
        self.0
            .parse(input, &recovery_point as &dyn Recognizer<I, E>)
    }
}
