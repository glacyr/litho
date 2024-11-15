use std::marker::PhantomData;

use nom::{Err, IResult};
use wrom::{terminal, Input, Recognizer, RecoverableParser};

use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue, Token};

use super::Error;

pub fn name<I, T>() -> impl RecoverableParser<I, Name<T>, Error>
where
    I: Input<Item = Token<T>> + Clone,
{
    terminal(|mut input: I| match input.next() {
        Some(Token::Name(name)) => Ok((input, name)),
        Some(_) => Err(Err::Error(Error::ExpectedName)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn name_unless<I, T>(unexpected: &'static str) -> impl RecoverableParser<I, Name<T>, Error>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Name(name)) if name.as_ref() != &unexpected => Ok((input, name)),
        Some(_) => Err(Err::Error(Error::ExpectedName)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn keyword<I, T>(expected: &'static str) -> impl RecoverableParser<I, Name<T>, Error>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Name(name)) if name.as_ref() == &expected => Ok((input, name)),
        Some(_) => Err(Err::Error(Error::ExpectedKeyword(expected))),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn punctuator<I, T>(expected: &'static str) -> impl RecoverableParser<I, Punctuator<T>, Error>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Punctuator(punctuator)) if punctuator.as_ref() == &expected => {
            Ok((input, punctuator))
        }
        Some(_) => Err(Err::Error(Error::ExpectedPunctuator(expected))),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn int_value<T, I>() -> impl RecoverableParser<I, IntValue<T>, Error>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::IntValue(int_value)) => Ok((input, int_value)),
        Some(_) => Err(Err::Error(Error::ExpectedIntValue)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn float_value<T, I>() -> impl RecoverableParser<I, FloatValue<T>, Error>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::FloatValue(float_value)) => Ok((input, float_value)),
        Some(_) => Err(Err::Error(Error::ExpectedFloatValue)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}

pub fn string_value<T, I>() -> impl RecoverableParser<I, StringValue<T>, Error>
where
    I: Input<Item = Token<T>> + Clone,
    T: for<'a> PartialEq<&'a str>,
{
    terminal(|mut input: I| match input.next() {
        Some(Token::StringValue(value)) => Ok((input, value)),
        Some(_) => Err(Err::Error(Error::ExpectedStringValue)),
        None => Err(Err::Error(Error::Incomplete)),
    })
}
