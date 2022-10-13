use nom::error::{ErrorKind, ParseError};
use nom::{Err, IResult};

pub fn next<I, E>(mut input: I) -> IResult<I, I::Item, E>
where
    I: Iterator,
    E: ParseError<I>,
{
    match input.next() {
        Some(token) => Ok((input, token)),
        None => Err(Err::Error(E::from_error_kind(input, ErrorKind::ManyTill))),
    }
}
