use std::ops::BitOr;

use enumset::EnumSet;
use wrom::{Input, RecoverableParser};

use crate::ast::{FloatValue, IntValue, Name, Punctuator, StringValue};
use crate::lex::{Token, TokenKind};

use super::Error;

#[derive(Clone, Copy, Debug, Default)]
pub struct RecoveryPoint {
    pub(crate) include: EnumSet<TokenKind>,
    pub(crate) exclude_on: bool,
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

impl BitOr for RecoveryPoint {
    type Output = RecoveryPoint;

    #[inline(always)]
    fn bitor(self, other: Self) -> Self {
        RecoveryPoint {
            include: self.include | other.include,
            exclude_on: (self.exclude_on || other.exclude_on)
                && (!self.include.contains(TokenKind::Name) || self.exclude_on)
                && (!other.include.contains(TokenKind::Name) || other.exclude_on),
        }
    }
}

macro_rules! token {
    ($($name:ident)*) => {
        $(
            impl<I, T> RecoverableParser<I, $name<T>, Error> for RecoveryPoint
            where
                I: Input<Recognizer = Self> + Iterator<Item = Token<T>>,
            {
                #[inline(always)]
                fn recognizer(&self) -> I::Recognizer {
                    *self
                }

                #[inline(always)]
                fn parse(&mut self, input: &mut I, _recovery_point: I::Recognizer) -> Result<$name<T>, Error> {
                    assert_eq!(input.recognize(*self), true);
                    // self.recognize(input)?;

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
