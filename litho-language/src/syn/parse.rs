use nom::Err;

use crate::lex::{lexer, SourceId, Token};

use super::{Error, Stream};

pub trait Parse<T>: Sized {
    fn parse<'a>(stream: Stream<'a, T>) -> Result<(Self, Vec<Token<T>>), Err<Error>>
    where
        T: From<&'a str> + Clone;

    fn parse_from_str<'a>(
        source_id: SourceId,
        input: &'a str,
    ) -> Result<(Self, Vec<Token<T>>), Err<Error>>
    where
        T: From<&'a str> + Clone,
    {
        let lexer = lexer(source_id, input);
        let stream = Stream::from(lexer);
        Self::parse(stream)
    }
}
