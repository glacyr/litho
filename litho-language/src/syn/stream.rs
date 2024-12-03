use std::marker::PhantomData;
use std::rc::Rc;

use bumpalo::boxed::Box;
use bumpalo::Bump;
use wrom::Input;

use crate::ast::{Context, ContextValue, List, Shared};
use crate::lex::{Lexer, Token, TokenKind};

use super::{BumpBox, RecoveryPoint, Span, Spanned};

pub struct Stream<'a, T>
where
    T: From<&'a str>,
{
    pub(crate) lexer: Lexer<'a, T>,
    bump: &'a Bump,
    unexpected: Vec<Token<'a, T>>,
}

impl<'a, T> Stream<'a, T>
where
    T: From<&'a str>,
{
    pub fn new(lexer: Lexer<'a, T>, bump: &'a Bump) -> Stream<'a, T> {
        Stream {
            lexer,
            bump,
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

impl<'a> Context<'a, &'a str> for Stream<'a, &'a str> {
    fn shared<U>(&self, value: U) -> Shared<'a, &'a str, U>
    where
        U: 'a,
    {
        Shared::new(BumpBox(self.bump.alloc(value) as *const _, PhantomData))
    }

    fn list<U>(&self) -> List<'a, &'a str, U>
    where
        U: Clone + 'a,
    {
        List::new(bumpalo::collections::Vec::new_in(&self.bump))
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

impl<'a, T> Iterator for Stream<'a, T>
where
    T: From<&'a str>,
{
    type Item = Token<'a, T>;

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
