use std::marker::PhantomData;

use logos::Lexer;

use super::{SourceId, Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct RawToken<T> {
    pub kind: TokenKind,
    pub source: T,
    pub span: Span,
}

impl<T> RawToken<T> {
    pub fn name(source: T) -> RawToken<T> {
        RawToken {
            kind: TokenKind::Name,
            source,
            span: Default::default(),
        }
    }

    pub fn punctuator(source: T) -> RawToken<T> {
        RawToken {
            kind: TokenKind::Punctuator,
            source,
            span: Default::default(),
        }
    }
}

impl<T> RawToken<T>
where
    T: PartialEq,
{
    pub fn congruent(&self, other: &Self) -> bool {
        self.kind == other.kind && self.source == other.source
    }
}

#[derive(Clone)]
pub struct RawLexer<'a, T> {
    source_id: SourceId,
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

pub fn raw_lexer<'a, T>(source_id: SourceId, lexer: Lexer<'a, TokenKind>) -> RawLexer<'a, T> {
    RawLexer {
        source_id,
        lexer,
        ty: PhantomData,
    }
}
