use wrom::Input;

use crate::lex::{Lexer, Token};

use super::{Span, Spanned};

pub struct Stream<'a, T>
where
    T: From<&'a str>,
{
    pub(crate) lexer: Lexer<'a, T>,
    unexpected: Vec<Token<T>>,
}

impl<'a, T> Stream<'a, T>
where
    T: From<&'a str>,
{
    pub fn into_unexpected<U>(self) -> U
    where
        U: FromIterator<Token<T>>,
    {
        self.lexer.chain(self.unexpected).collect()
    }
}

impl<'a, T> Spanned for Stream<'a, T>
where
    T: From<&'a str>,
{
    #[inline(always)]
    fn span(&mut self) -> Span {
        self.lexer.span()
    }
}

impl<'a, T> From<Lexer<'a, T>> for Stream<'a, T>
where
    T: From<&'a str>,
{
    fn from(lexer: Lexer<'a, T>) -> Self {
        Stream {
            lexer,
            unexpected: vec![],
        }
    }
}

impl<'a, T> Input for Stream<'a, T>
where
    T: From<&'a str>,
{
    type Item = Token<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }

    #[inline(always)]
    fn peek(&mut self) -> Option<&Self::Item> {
        self.lexer.peek()
    }

    #[inline(always)]
    fn unrecognized(&mut self, item: Self::Item) {
        self.unexpected.push(item)
    }
}
