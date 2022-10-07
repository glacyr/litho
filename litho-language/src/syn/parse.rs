use nom::Err;

use crate::lex::{lexer, Token};

use super::{Error, Stream};

pub trait Parse<T>: Sized {
    fn parse(stream: Stream<T>) -> Result<(Self, Vec<Token<T>>), Err<Error>>;

    fn parse_from_str<'a>(
        source_id: usize,
        input: &'a str,
    ) -> Result<(Self, Vec<Token<T>>), Err<Error>>
    where
        T: From<&'a str>,
    {
        Self::parse(Stream::from(lexer(source_id, input).exact()))
    }
}
