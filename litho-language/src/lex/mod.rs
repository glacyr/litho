mod kind;
mod raw;
mod token;

pub use kind::TokenKind;
pub use raw::{raw_lexer, RawLexer, RawToken};
pub use token::{lexer, Lexer, Name, Punctuator, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub source_id: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn join(&mut self, other: Span) {
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }

    pub fn collapse_to_end(self) -> Self {
        Self {
            source_id: self.source_id,
            start: self.end,
            end: self.end,
        }
    }
}

impl ariadne::Span for Span {
    type SourceId = usize;

    fn source(&self) -> &Self::SourceId {
        &self.source_id
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}
