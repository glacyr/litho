use std::marker::PhantomData;

use logos::Lexer;

use super::{SourceId, Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct RawToken<'a, T> {
    pub kind: TokenKind,
    pub source: T,
    pub span: Span,
    pub marker: PhantomData<&'a ()>,
}

impl<'a, T> RawToken<'a, T> {
    pub fn new(kind: TokenKind, source: T) -> RawToken<'a, T> {
        RawToken {
            kind,
            source,
            span: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<'a, T> RawToken<'a, T>
where
    T: PartialEq,
{
    pub fn congruent(&self, other: &Self) -> bool {
        self.kind == other.kind && self.source == other.source
    }
}

#[derive(Clone)]
pub struct RawLexer<'a, 'b, T> {
    source_id: SourceId,
    lexer: Lexer<'b, TokenKind>,
    ty: PhantomData<&'a T>,
}

impl<'a, 'b, T> Iterator for RawLexer<'a, 'b, T>
where
    T: From<&'b str>,
{
    type Item = RawToken<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|kind| RawToken {
            kind,
            source: T::from(self.lexer.slice()),
            span: Span {
                source_id: self.source_id,
                start: self.lexer.span().start,
                end: self.lexer.span().end,
            },
            marker: PhantomData,
        })
    }
}

pub fn raw_lexer<'a, 'b, T>(
    source_id: SourceId,
    lexer: Lexer<'b, TokenKind>,
) -> RawLexer<'a, 'b, T> {
    RawLexer {
        source_id,
        lexer,
        ty: PhantomData,
    }
}
