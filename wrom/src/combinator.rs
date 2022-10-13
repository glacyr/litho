use nom::error::ParseError;
use nom::multi::many_till;
use nom::{IResult, Parser};

use super::{next, Input, Recognizer, RecoverableParser};

pub struct Opt<P>(P);

impl<I, E, P> Recognizer<I, E> for Opt<P>
where
    P: Recognizer<I, E>,
{
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
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, Option<O>, E>
    where
        R: Recognizer<I, E>,
    {
        let input_ = input.clone();

        let (mut input, (rest, value)) = many_till(
            next,
            (|input| self.0.parse(input, recovery_point.by_ref()))
                .map(Some)
                .or({ |input| recovery_point.recognize(input) }.map(|_| None)),
        )
        .parse(input)?;

        match value {
            Some(value) => {
                input.extend(rest);
                Ok((input, Some(value)))
            }
            None => Ok((input_, None)),
        }
    }
}

pub fn opt<P>(parser: P) -> Opt<P> {
    Opt(parser)
}
