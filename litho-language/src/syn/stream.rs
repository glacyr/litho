use nom::InputLength;
use wrom::Input;

use crate::lex::{ExactLexer, Token};

use super::{Missing, MissingToken};

#[derive(Clone)]
pub struct Stream<'a> {
    lexer: ExactLexer<'a>,
    unexpected: Vec<Token<'a>>,
}

impl<'a> Stream<'a> {
    pub fn into_unexpected<U>(self) -> U
    where
        U: FromIterator<Token<'a>>,
    {
        self.lexer.chain(self.unexpected).collect()
    }
}

impl<'a> From<ExactLexer<'a>> for Stream<'a> {
    fn from(lexer: ExactLexer<'a>) -> Self {
        Stream {
            lexer,
            unexpected: vec![],
        }
    }
}

impl<'a> Iterator for Stream<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}

impl<'a> Extend<Token<'a>> for Stream<'a> {
    fn extend<T: IntoIterator<Item = Token<'a>>>(&mut self, iter: T) {
        self.unexpected.extend(iter)
    }
}

impl<'a> InputLength for Stream<'a> {
    fn input_len(&self) -> usize {
        self.lexer.input_len()
    }
}

impl<'a> Input for Stream<'a> {
    type Item = Token<'a>;
    type Missing = Missing;

    fn missing(&self, missing: Missing) -> MissingToken {
        MissingToken {
            span: self.lexer.span(),
            missing,
        }
    }
}
