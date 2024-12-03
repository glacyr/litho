use bumpalo::Bump;

use crate::ast::{Context, ContextValue};
use crate::lex::{lexer, SourceId, Token};

use super::{Error, Stream};

pub trait Parse<'a, T>: Sized {
    fn parse(stream: Stream<'a, T>) -> Result<(Self, Vec<Token<T>>), Error>
    where
        Stream<'a, T>: Context<'a, T>,
        T: ContextValue<'a> + From<&'a str> + 'a;

    fn parse_from_str(
        source_id: SourceId,
        input: &'a str,
        bump: &'a Bump,
    ) -> Result<(Self, Vec<Token<'a, T>>), Error>
    where
        Stream<'a, T>: Context<'a, T>,
        T: ContextValue<'a> + From<&'a str> + 'a,
    {
        let lexer = lexer(source_id, input);
        let stream = Stream::new(lexer, bump);
        Self::parse(stream)
    }
}
