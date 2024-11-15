use nom::Err;
use wrom::{terminal, Input, RecoverableParser};

use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue, Token, TokenKind};

use super::recovery::RecoveryPoint;
use super::Error;

pub fn name<I, T>() -> impl RecoverableParser<I, Name<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>> + Clone,
{
    terminal(TokenKind::Name.into(), |mut input: I| match input.next() {
        Some(Token::Name(name)) => Ok((input, name)),
        Some(_) => Err(Err::Error(Error::ExpectedName)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn name_unless<I, T>(
    unexpected: TokenKind,
) -> impl RecoverableParser<I, Name<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(TokenKind::Name.into(), move |mut input: I| {
        match input.next() {
            Some(Token::Name(name)) if name.as_raw_token().kind != unexpected => Ok((input, name)),
            Some(_) => Err(Err::Error(Error::ExpectedName)),
            None => Err(Err::Error(Error::Incomplete)),
        }
    })
}

pub fn keyword<I, T>(
    expected: TokenKind,
) -> impl RecoverableParser<I, Name<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>> + Clone,
{
    terminal(expected.into(), move |mut input: I| match input.next() {
        Some(Token::Name(name)) if name.as_raw_token().kind == expected => Ok((input, name)),
        Some(_) => Err(Err::Error(Error::ExpectedKeyword(expected))),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn punctuator<I, T>(
    expected: TokenKind,
) -> impl RecoverableParser<I, Punctuator<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>>,
{
    terminal(expected.into(), move |mut input: I| match input.next() {
        Some(Token::Punctuator(punctuator)) if punctuator.as_raw_token().kind == expected => {
            Ok((input, punctuator))
        }
        Some(_) => Err(Err::Error(Error::ExpectedPunctuator(expected))),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn int_value<T, I>() -> impl RecoverableParser<I, IntValue<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(
        TokenKind::IntValue.into(),
        move |mut input: I| match input.next() {
            Some(Token::IntValue(int_value)) => Ok((input, int_value)),
            Some(_) => Err(Err::Error(Error::ExpectedIntValue)),
            None => Err(Err::Error(Error::Incomplete)),
        },
    )
}

pub fn float_value<T, I>() -> impl RecoverableParser<I, FloatValue<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(
        TokenKind::FloatValue.into(),
        move |mut input: I| match input.next() {
            Some(Token::FloatValue(float_value)) => Ok((input, float_value)),
            Some(_) => Err(Err::Error(Error::ExpectedFloatValue)),
            None => Err(Err::Error(Error::Incomplete)),
        },
    )
}

pub fn string_value<T, I>() -> impl RecoverableParser<I, StringValue<T>, Error, RecoveryPoint>
where
    I: Input<Item = Token<T>> + Clone,
{
    terminal(TokenKind::StringValue.into(), |mut input: I| {
        match input.next() {
            Some(Token::StringValue(value)) => Ok((input, value)),
            Some(_) => Err(Err::Error(Error::ExpectedStringValue)),
            None => Err(Err::Error(Error::Incomplete)),
        }
    })
}
