use nom::error::ParseError;
use nom::Parser;

use super::{Input, Recognizer, RecoverableParser};

pub struct Many<P>(P, bool);

impl<I, E, P> Recognizer<I, E> for Many<P>
where
    P: Recognizer<I, E>,
{
    fn recognizer(&self) -> impl Parser<I, (), E> {
        self.0.recognizer()
    }
}

impl<I, O, E, P> RecoverableParser<I, Vec<O>, E> for Many<P>
where
    I: Clone + Input,
    P: RecoverableParser<I, O, E>,
    E: ParseError<I>,
{
    fn parser<R>(&self, recovery_point: R) -> impl Parser<I, Vec<O>, E>
    where
        R: Recognizer<I, E>,
    {
        move |input: I| {
            let parser = self.0.parser(recovery_point.by_ref().or(&self.0));

            match self.1 {
                false => nom::multi::many0(parser).parse(input),
                true => nom::multi::many1(parser).parse(input),
            }
        }
    }
}

pub fn many0<P>(parser: P) -> Many<P> {
    Many(parser, false)
}

pub fn many1<P>(parser: P) -> Many<P> {
    Many(parser, true)
}
