use nom::InputLength;
use wrom::Input;

use crate::lex::{ExactLexer, Token};

use super::{Missing, MissingToken};

#[derive(Clone)]
pub struct Stream<T> {
    lexer: ExactLexer<T>,
    unexpected: Vec<Token<T>>,
}

impl<T> Stream<T> {
    pub fn into_unexpected<U>(self) -> U
    where
        U: FromIterator<Token<T>>,
    {
        self.lexer.chain(self.unexpected).collect()
    }
}

impl<T> From<ExactLexer<T>> for Stream<T> {
    fn from(lexer: ExactLexer<T>) -> Self {
        Stream {
            lexer,
            unexpected: vec![],
        }
    }
}

impl<T> Iterator for Stream<T> {
    type Item = Token<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}

impl<T> Extend<Token<T>> for Stream<T> {
    fn extend<I: IntoIterator<Item = Token<T>>>(&mut self, iter: I) {
        self.unexpected.extend(iter)
    }
}

impl<T> InputLength for Stream<T> {
    fn input_len(&self) -> usize {
        self.lexer.input_len()
    }
}

impl<T> Input for Stream<T>
where
    T: Clone,
{
    type Item = Token<T>;
    type Missing = Missing;

    fn missing(&self, missing: Missing) -> MissingToken {
        MissingToken {
            span: self.lexer.span(),
            missing,
        }
    }
}
