use nom::Err;

use crate::lex::{lexer, Token};

use super::{Error, Stream};

pub trait Parse<'a>: Sized {
    fn parse(stream: Stream<'a>) -> Result<(Self, Vec<Token<'a>>), Err<Error>>;

    fn parse_from_str(
        source_id: usize,
        input: &'a str,
    ) -> Result<(Self, Vec<Token<'a>>), Err<Error>> {
        Self::parse(Stream::from(lexer(source_id, input).exact()))
    }
}
