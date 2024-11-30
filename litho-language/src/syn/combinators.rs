use wrom::{Input, RecoverableParser};
use wrom_derive::wrom;

use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue, Token, TokenKind};

use super::recovery::RecoveryPoint;
use super::Error;

#[inline]
pub fn name<I, T>() -> impl RecoverableParser<I, Name<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    RecoveryPoint::from(TokenKind::Name)
}

#[inline]
pub fn name_unless_on<I, T>() -> impl RecoverableParser<I, Name<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    RecoveryPoint::name_unless_on()
}

#[wrom]
pub fn keyword<I, T>(
    expected: TokenKind,
) -> impl RecoverableParser<I, Name<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    RecoveryPoint::from(expected)
}

#[wrom]
pub fn punctuator<I, T>(
    expected: TokenKind,
) -> impl RecoverableParser<I, Punctuator<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    RecoveryPoint::from(expected)
}

#[wrom]
pub fn int_value<T, I>() -> impl RecoverableParser<I, IntValue<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    RecoveryPoint::from(TokenKind::IntValue)
}

#[wrom]
pub fn float_value<T, I>() -> impl RecoverableParser<I, FloatValue<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    RecoveryPoint::from(TokenKind::FloatValue)
}

#[wrom]
pub fn string_value<T, I>() -> impl RecoverableParser<I, StringValue<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    RecoveryPoint::from(TokenKind::StringValue)
}
