use nom::error::{ErrorKind, ParseError};
use nom::multi::many_till;
use nom::IResult;
use nom::Parser;

use super::{next, Input, Recognizer, RecoverableParser};

pub struct Many0<P>(P);

pub fn many0<P>(parser: P) -> Many0<P> {
    Many0(parser)
}

impl<I, E, P> Recognizer<I, E> for Many0<P>
where
    P: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O, E, P> RecoverableParser<I, Vec<O>, E> for Many0<P>
where
    I: Clone + Input,
    P: RecoverableParser<I, O, E>,
    E: ParseError<I>,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, Vec<O>, E>
    where
        R: Recognizer<I, E>,
    {
        let mut input = input;
        let mut items = vec![];

        loop {
            match many_till(
                next,
                (|input| self.0.parse(input, recovery_point.by_ref().or(&self.0)))
                    .map(Some)
                    .or({ |input| recovery_point.recognize(input) }.map(|_| None)),
            )
            .parse(input.clone())
            {
                Ok((input_, (rest, Some(item)))) => {
                    input = input_;
                    input.extend(rest);
                    items.push(item);
                }
                Ok(_) => {
                    return Ok((input, items));
                }
                Err(_) => return Ok((input, items)),
            }
        }
    }
}

pub struct Many1<P>(P);

pub fn many1<P>(parser: P) -> Many1<P> {
    Many1(parser)
}

impl<I, E, P> Recognizer<I, E> for Many1<P>
where
    P: Recognizer<I, E>,
{
    fn recognize(&self, input: I) -> IResult<I, (), E> {
        self.0.recognize(input)
    }
}

impl<I, O, E, P> RecoverableParser<I, Vec<O>, E> for Many1<P>
where
    I: Clone + Input,
    P: RecoverableParser<I, O, E>,
    E: ParseError<I>,
{
    fn parse<R>(&self, input: I, recovery_point: R) -> IResult<I, Vec<O>, E>
    where
        R: Recognizer<I, E>,
    {
        let mut input = input;
        let mut items = vec![];

        loop {
            match many_till(
                next,
                (|input| self.0.parse(input, recovery_point.by_ref().or(&self.0)))
                    .map(Some)
                    .or({ |input| recovery_point.recognize(input) }.map(|_| None)),
            )
            .parse(input.clone())
            {
                Ok((input_, (rest, Some(item)))) => {
                    input = input_;
                    input.extend(rest);
                    items.push(item);
                }
                _ if !items.is_empty() => {
                    return Ok((input, items));
                }
                _ => return Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Many1))),
            }
        }
    }
}
