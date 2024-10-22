use nom::error::{ErrorKind, ParseError};
use nom::multi::many_till;
use nom::{Err, IResult, Parser};

use super::{Input, Recognizer, RecoverableParser};

fn next<I, E>(mut input: I) -> IResult<I, I::Item, E>
where
    I: Iterator,
    E: ParseError<I>,
{
    match input.next() {
        Some(token) => Ok((input, token)),
        None => Err(Err::Error(E::from_error_kind(input, ErrorKind::ManyTill))),
    }
}

fn extend_rest<P, I, O, E>(mut parser: P) -> impl Parser<I, Option<O>, E>
where
    P: Parser<I, (Vec<<I as Iterator>::Item>, Option<O>), E>,
    I: Clone + Input,
    E: ParseError<I>,
{
    move |input: I| {
        let (mut input, (rest, value)) = parser.parse(input)?;
        if value.is_some() {
            input.extend(rest);
        }
        Ok((input, value))
    }
}

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
        move |input: I| {
            extend_rest(many_till(
                next,
                // This parser returns:
                // - Ok(Some(_)) if it has parsed something
                // - Ok(None)    if it has reached the recovery point
                // - Err(_)      along the way
                self.0
                    .parser(&recovery_point)
                    .map(Some)
                    .or(recovery_point.recognizer().map(|_| None)),
            ))
            .parse(input)
        }
    }
}

pub fn opt<P>(parser: P) -> Opt<P> {
    Opt(parser)
}
