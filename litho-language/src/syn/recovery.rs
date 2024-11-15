use enumset::{EnumSet, EnumSetType};
use nom::{Err, IResult};
use wrom::{Input, Recognizer};

use crate::lex::{Token, TokenKind};

use super::Error;

#[derive(Clone, Copy, Debug, Default)]
pub struct RecoveryPoint(EnumSet<TokenKind>);

impl From<TokenKind> for RecoveryPoint {
    fn from(value: TokenKind) -> Self {
        RecoveryPoint(value.into())
    }
}

impl<I, T> Recognizer<I, Error> for RecoveryPoint
where
    I: Input<Item = Token<T>>,
{
    fn recognize(self, mut input: I) -> IResult<I, (), Error> {
        match input.next() {
            Some(token) if self.0.contains(token.as_raw_token().kind) => Ok((input, ())),
            Some(Token::Name(_)) if self.0.contains(TokenKind::Name) => Ok((input, ())),
            _ => Err(Err::Error(Error::ExpectedName)),
        }
    }

    fn or(self, other: Self) -> Self {
        RecoveryPoint(self.0.union(other.0))
    }
}

#[derive(EnumSetType)]
pub enum Keyword {
    Type,
}
