use wrom::Input;

use crate::ast::{Context, ContextValue, List, Shared};
use crate::lex::{Lexer, Token, TokenKind};

use super::{RecoveryPoint, Span, Spanned};

pub struct Stream<'a, 'b, T, C>
where
    T: ContextValue<'a> + From<&'b str>,
    C: Context<'a, T>,
{
    pub(crate) lexer: Lexer<'a, 'b, T>,
    context: C,
    unexpected: Vec<Token<'a, T>>,
}

impl<'a, 'b, T, C> Stream<'a, 'b, T, C>
where
    T: ContextValue<'a> + From<&'b str>,
    C: Context<'a, T>,
{
    pub fn new(lexer: Lexer<'a, 'b, T>, context: C) -> Stream<'a, 'b, T, C> {
        Stream {
            lexer,
            context,
            unexpected: Default::default(),
        }
    }

    pub fn into_unexpected<U>(self) -> U
    where
        U: FromIterator<Token<'a, T>>,
    {
        self.lexer.chain(self.unexpected).collect()
    }
}

impl<'a, 'b, T, C> Context<'a, T> for Stream<'a, 'b, T, C>
where
    T: ContextValue<'a> + From<&'b str>,
    C: Context<'a, T>,
{
    fn shared<U>(&self, value: U) -> Shared<'a, T, U>
    where
        U: 'a,
    {
        self.context.shared(value)
    }

    fn list<U>(&self) -> List<'a, T, U>
    where
        U: Clone + 'a,
    {
        self.context.list()
    }

    fn list_from_iter<'c, I>(&self, iter: I) -> List<'a, T, I::Item>
    where
        I: Iterator + 'c,
        I::Item: Clone + 'a,
    {
        self.context.list_from_iter(iter)
    }
}

impl<'a, 'b, T, C> Spanned for Stream<'a, 'b, T, C>
where
    T: ContextValue<'a> + From<&'b str>,
    C: Context<'a, T>,
{
    #[inline(always)]
    fn span(&mut self) -> Span {
        self.lexer.span()
    }
}

impl<'a, 'b, T, C> Iterator for Stream<'a, 'b, T, C>
where
    T: ContextValue<'a> + From<&'b str>,
    C: Context<'a, T>,
{
    type Item = Token<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}

impl<'a, 'b, T, C> Input for Stream<'a, 'b, T, C>
where
    T: ContextValue<'a> + From<&'b str>,
    C: Context<'a, T>,
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
