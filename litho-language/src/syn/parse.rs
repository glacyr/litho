use crate::ast::{Context, ContextValue};
use crate::lex::{lexer, SourceId, Token};

use super::{Error, Stream};

pub trait Parse<'a, T>: Sized {
    fn parse<'b, C>(stream: Stream<'a, 'b, T, C>) -> Result<(Self, Vec<Token<'a, T>>), Error>
    where
        T: ContextValue<'a> + From<&'b str> + 'a,
        C: Context<'a, T> + 'a;

    fn parse_from_str<'b, C>(
        source_id: SourceId,
        input: &'b str,
        context: C,
    ) -> Result<(Self, Vec<Token<'a, T>>), Error>
    where
        T: ContextValue<'a> + From<&'b str> + 'a,
        C: Context<'a, T> + 'a,
    {
        let lexer = lexer(source_id, input);
        let stream = Stream::new(lexer, context);
        Self::parse(stream)
    }
}
