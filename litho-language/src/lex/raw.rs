use logos::Lexer;

use super::{Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct RawToken<'a> {
    pub kind: TokenKind,
    pub source: &'a str,
    pub span: Span,
}

#[derive(Clone)]
pub struct RawLexer<'a> {
    source_id: usize,
    lexer: Lexer<'a, TokenKind>,
}

impl<'a> Iterator for RawLexer<'a> {
    type Item = RawToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|kind| RawToken {
            kind,
            source: self.lexer.slice(),
            span: Span {
                source_id: self.source_id,
                start: self.lexer.span().start,
                end: self.lexer.span().end,
            },
        })
    }
}

pub fn raw_lexer<'a>(source_id: usize, lexer: Lexer<'a, TokenKind>) -> RawLexer<'a> {
    RawLexer { source_id, lexer }
}
