use wrom::Input;

use crate::lex::{Lexer, Token, TokenKind};

use super::{RecoveryPoint, Span, Spanned};

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

impl<'a, T> Iterator for Stream<'a, T>
where
    T: From<&'a str>,
{
    type Item = Token<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}

impl<'a, T> Input for Stream<'a, T>
where
    T: From<&'a str>,
{
    type Recognizer = RecoveryPoint;

    #[inline(always)]
    fn recognize(&mut self, recognizer: Self::Recognizer) -> bool {
        match self.lexer.peek() {
            Some(token) if recognizer.include.contains(token.as_raw_token().kind) => true,
            Some(Token::Name(name))
                if recognizer.include.contains(TokenKind::Name)
                    && (!recognizer.exclude_on
                        || name.as_raw_token().kind != TokenKind::KeywordOn) =>
            {
                true
            }
            _ => false,
        }
    }

    #[inline(always)]
    fn discard(&mut self, recognizer: Self::Recognizer, recovery_point: Self::Recognizer) -> bool {
        loop {
            if self.recognize(recognizer) {
                break true;
            } else if self.recognize(recovery_point) {
                break false;
            } else if let Some(token) = self.lexer.next() {
                self.unexpected.push(token);

                continue;
            } else {
                break false;
            }
        }
    }
}
