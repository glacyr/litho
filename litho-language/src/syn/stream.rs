use nom::InputLength;
use wrom::Input;

use crate::lex::{ExactLexer, FastLexer, Token};

use super::{Missing, MissingToken};

#[derive(Clone)]
pub struct Stream<'a, T> {
    lexer: FastLexer<'a, T>,
    unexpected: Vec<Token<T>>,
}

impl<'a, T> Stream<'a, T> {
    pub fn into_unexpected<U>(self) -> U
    where
        T: Clone,
        U: FromIterator<Token<T>>,
    {
        self.lexer.chain(self.unexpected).collect()
    }
}

impl<'a, T> From<&'a ExactLexer<T>> for Stream<'a, T> {
    fn from(lexer: &'a ExactLexer<T>) -> Self {
        Stream {
            lexer: FastLexer::new(lexer.tokens.as_slices().0),
            unexpected: vec![],
        }
    }
}

impl<'a, T> Iterator for Stream<'a, T>
where
    T: Clone,
{
    type Item = Token<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}

impl<'a, T> Extend<Token<T>> for Stream<'a, T> {
    fn extend<I: IntoIterator<Item = Token<T>>>(&mut self, iter: I) {
        self.unexpected.extend(iter)
    }
}

impl<'a, T> InputLength for Stream<'a, T> {
    fn input_len(&self) -> usize {
        self.lexer.input_len()
    }
}

impl<'a, T> Input for Stream<'a, T>
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
