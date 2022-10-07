use std::marker::PhantomData;

use logos::Lexer;

use super::{Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct RawToken<T> {
    pub kind: TokenKind,
    pub source: T,
    pub span: Span,
}

#[derive(Clone)]
pub struct RawLexer<'a, T> {
    source_id: usize,
    lexer: Lexer<'a, TokenKind>,
    ty: PhantomData<T>,
}

impl<'a, T> Iterator for RawLexer<'a, T>
where
    T: From<&'a str>,
{
    type Item = RawToken<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|kind| RawToken {
            kind,
            source: T::from(self.lexer.slice()),
            span: Span {
                source_id: self.source_id,
                start: self.lexer.span().start,
                end: self.lexer.span().end,
            },
        })
    }
}

pub fn raw_lexer<'a, T>(source_id: usize, lexer: Lexer<'a, TokenKind>) -> RawLexer<'a, T> {
    RawLexer {
        source_id,
        lexer,
        ty: PhantomData,
    }
}
