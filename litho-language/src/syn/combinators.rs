use wrom::{Input, RecoverableParser};
use wrom_derive::wrom;

use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue, Token, TokenKind};

use super::recovery::RecoveryPoint;
use super::Error;

#[wrom]
pub fn name<'a, I, T>() -> impl RecoverableParser<I, Name<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>>,
{
    RecoveryPoint::from(TokenKind::Name)
}

#[wrom]
pub fn name_unless_on<'a, I, T>() -> impl RecoverableParser<I, Name<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>>,
{
    RecoveryPoint::name_unless_on()
}

#[wrom]
pub fn keyword<'a, I, T>(expected: TokenKind) -> impl RecoverableParser<I, Name<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>>,
{
    RecoveryPoint::from(expected)
}

#[wrom]
pub fn punctuator<'a, I, T>(
    expected: TokenKind,
) -> impl RecoverableParser<I, Punctuator<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>>,
{
    RecoveryPoint::from(expected)
}

#[wrom]
pub fn int_value<'a, I, T>() -> impl RecoverableParser<I, IntValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>>,
{
    RecoveryPoint::from(TokenKind::IntValue)
}

#[wrom]
pub fn float_value<'a, I, T>() -> impl RecoverableParser<I, FloatValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>>,
{
    RecoveryPoint::from(TokenKind::FloatValue)
}

#[wrom]
pub fn string_value<'a, I, T>() -> impl RecoverableParser<I, StringValue<'a, T>, Error>
where
    I: Input<Recognizer = RecoveryPoint> + Iterator<Item = Token<'a, T>>,
{
    RecoveryPoint::from(TokenKind::StringValue)
}
