use enumset::EnumSet;
use wrom::{Input, Recognizer, RecoverableParser};

use crate::ast::{FloatValue, IntValue, Name, Punctuator, StringValue};
use crate::lex::{Token, TokenKind};

use super::Error;

#[derive(Clone, Copy, Debug, Default)]
pub struct RecoveryPoint {
    include: EnumSet<TokenKind>,
    exclude_on: bool,
}

impl From<TokenKind> for RecoveryPoint {
    #[inline]
    fn from(value: TokenKind) -> Self {
        RecoveryPoint {
            include: value.into(),
            exclude_on: false,
        }
    }
}

impl RecoveryPoint {
    #[inline]
    pub fn name_unless_on() -> RecoveryPoint {
        RecoveryPoint {
            include: TokenKind::Name.into(),
            exclude_on: true,
        }
    }
}

impl<I, T> Recognizer<I, Error> for RecoveryPoint
where
    I: Input<Item = Token<T>>,
{
    #[inline(always)]
    fn recognize(self, input: &mut I) -> Result<(), Error> {
        match input.peek() {
            Some(token) if self.include.contains(token.as_raw_token().kind) => Ok(()),
            Some(Token::Name(name))
                if self.include.contains(TokenKind::Name)
                    && (!self.exclude_on || name.as_raw_token().kind != TokenKind::KeywordOn) =>
            {
                Ok(())
            }
            _ => Err(Error::Expected(self)),
        }
    }

    #[inline(always)]
    fn or(self, other: Self) -> Self {
        RecoveryPoint {
            include: self.include.union(other.include),
            exclude_on: (self.exclude_on || other.exclude_on)
                && (!self.include.contains(TokenKind::Name) || self.exclude_on)
                && (!other.include.contains(TokenKind::Name) || other.exclude_on),
        }
    }
}

macro_rules! token {
    ($($name:ident)*) => {
        $(
            impl<I, T> RecoverableParser<I, $name<T>, Error, RecoveryPoint> for RecoveryPoint
            where
                I: Input<Item = Token<T>>,
            {
                #[inline(always)]
                fn recognizer(&self) -> RecoveryPoint {
                    *self
                }

                #[inline(always)]
                fn parse(&mut self, input: &mut I, _recovery_point: RecoveryPoint) -> Result<$name<T>, Error> {
                    self.recognize(input)?;

                    match input.next() {
                        Some(Token::$name(token)) => Ok(token),
                        _ => Err(Error::Expected(*self)),
                    }
                }
            }
        )*
    }
}

token!(Name Punctuator IntValue FloatValue StringValue);
