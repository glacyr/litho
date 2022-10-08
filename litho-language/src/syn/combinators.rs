use nom::Err;
use wrom::{terminal, RecoverableParser};

use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue, Token};

use super::Error;

pub fn name<T, I>() -> impl RecoverableParser<I, Name<T>, Error>
where
    I: Iterator<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Name(name)) if name.as_ref() != &"fragment" => Ok((input, name)),
        Some(token) => Err(Err::Error(Error::ExpectedName)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn keyword<T, I>(expected: &'static str) -> impl RecoverableParser<I, Name<T>, Error>
where
    I: Iterator<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Name(name)) if name.as_ref() == &expected => Ok((input, name)),
        Some(token) => Err(Err::Error(Error::ExpectedKeyword(expected))),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn punctuator<T, I>(expected: &'static str) -> impl RecoverableParser<I, Punctuator<T>, Error>
where
    I: Iterator<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Punctuator(actual)) if actual.as_ref() == &expected => Ok((input, actual)),
        Some(token) => Err(Err::Error(Error::ExpectedPunctuator(expected))),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn int_value<T, I>() -> impl RecoverableParser<I, IntValue<T>, Error>
where
    I: Iterator<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::IntValue(int_value)) => Ok((input, int_value)),
        Some(token) => Err(Err::Error(Error::ExpectedIntValue)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn float_value<T, I>() -> impl RecoverableParser<I, FloatValue<T>, Error>
where
    I: Iterator<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::FloatValue(float_value)) => Ok((input, float_value)),
        Some(token) => Err(Err::Error(Error::ExpectedFloatValue)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn string_value<T, I>() -> impl RecoverableParser<I, StringValue<T>, Error>
where
    I: Iterator<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::StringValue(string_value)) => Ok((input, string_value)),
        Some(token) => Err(Err::Error(Error::ExpectedStringValue)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}
