use std::iter::once;

use nom::combinator::{cut, eof, fail, opt, peek};
use nom::multi::{many0, many_till};
use nom::sequence::tuple;
use nom::{IResult, InputLength, Parser};
use wrom::{terminal, RecoverableParser};

use crate::ast::*;
use crate::lex::{Name, Punctuator, Token};

use super::Error;

// pub fn name_if<'a, I>(expected: &'static str) -> impl Fn(I) -> IResult<I, Name<'a>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     move |input| {
//         let (input, name) = Name::parse(input)?;

//         match name.as_ref() == expected {
//             true => Ok((input, name)),
//             false => Err(nom::Err::Error(Error::ExpectedName(
//                 name.span(),
//                 vec![expected],
//             ))),
//         }
//     }
// }

// pub fn name_if_fn<'a, F, I>(mut condition: F) -> impl FnMut(I) -> IResult<I, Name<'a>, Error<'a>>
// where
//     F: FnMut(&str) -> bool,
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     move |input| {
//         let (input, name) = Name::parse(input)?;

//         match condition(name.as_ref()) {
//             true => Ok((input, name)),
//             false => Err(nom::Err::Error(Error::ExpectedName(name.span(), vec![]))),
//         }
//     }
// }

pub fn name<'a, I>() -> impl RecoverableParser<I, Name<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Name(name)) => Ok((input, name)),
        Some(token) => Err(nom::Err::Error(Error::ExpectedName(token.span(), vec![]))),
        None => Err(nom::Err::Error(Error::Incomplete)),
    })
}

pub fn keyword<'a, I>(expected: &'static str) -> impl RecoverableParser<I, Name<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Name(name)) if name.as_ref() == expected => Ok((input, name)),
        Some(token) => Err(nom::Err::Error(Error::ExpectedName(
            token.span(),
            vec![expected],
        ))),
        None => Err(nom::Err::Error(Error::Incomplete)),
    })
}

pub fn punctuator<'a, I>(
    expected: &'static str,
) -> impl RecoverableParser<I, Punctuator<'a>, Error<'a>>
where
    I: Iterator<Item = Token<'a>> + Clone,
{
    terminal(move |mut input: I| match input.next() {
        Some(Token::Punctuator(actual)) if actual.as_ref() == expected => Ok((input, actual)),
        Some(token) => Err(nom::Err::Error(Error::ExpectedName(
            token.span(),
            vec![expected],
        ))),
        None => Err(nom::Err::Error(Error::Incomplete)),
    })
}

// pub fn punctuator_if<'a, I>(
//     expected: &'static str,
// ) -> impl Fn(I) -> IResult<I, Punctuator<'a>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     move |input| {
//         let (input, punctuator) = Punctuator::parse(input)?;

//         match punctuator.as_ref() == expected {
//             true => Ok((input, punctuator)),
//             false => Err(nom::Err::Error(Error::ExpectedName(
//                 punctuator.span(),
//                 vec![expected],
//             ))),
//         }
//     }
// }

// pub fn delimiter<'a, I>(input: I) -> IResult<I, Punctuator<'a>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     let (input, punctuator) = Punctuator::parse(input)?;

//     match punctuator.as_ref() {
//         "{" | "(" | "[" | "]" | ")" | "}" => Ok((input, punctuator)),
//         _ => Err(nom::Err::Error(Error::ExpectedName(
//             punctuator.span(),
//             vec![],
//         ))),
//     }
// }

// pub fn lower_delimiter<'a, I>(scope: &'static str) -> impl Parser<I, Punctuator<'a>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     move |input| {
//         let (input, punctuator) = Punctuator::parse(input)?;

//         match (scope, punctuator.as_ref()) {
//             ("{", "}") | ("(", "{" | ")" | "}") => Ok((input, punctuator)),
//             _ => Err(nom::Err::Error(Error::ExpectedName(
//                 punctuator.span(),
//                 vec![],
//             ))),
//         }
//     }
// }

// pub fn lower_delimiter_or_eof<'a, I>(scope: &'static str) -> impl Parser<I, (), Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     peek(lower_delimiter(scope).map(|_| ()).or(eof.map(|_| ())))
// }

// pub fn sequence_point<'a, I>(input: I) -> IResult<I, Punctuator<'a>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     let (input, punctuator) = Punctuator::parse(input)?;

//     match punctuator.as_ref() {
//         "{" | "(" | ")" | "}" | "@" | ":" | "=" | "$" | "[" | "]" => Ok((input, punctuator)),
//         _ => Err(nom::Err::Error(Error::ExpectedName(
//             punctuator.span(),
//             vec![],
//         ))),
//     }
// }

// pub fn recovery_point<'a, I>(input: I) -> IResult<I, (), Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     Name::parse
//         .map(|_| ())
//         .or(sequence_point.map(|_| ()))
//         .or(eof.map(|_| ()))
//         .parse(input)
// }

// pub fn recoverable_punctuator_if<'a, I>(
//     expected: &'static str,
// ) -> impl Parser<I, Recoverable<'a, Punctuator<'a>>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     opt(punctuator_if(expected)).map(|punctuator| punctuator.ok_or(vec![]))
// }

// pub fn group<'a, T, I>(
//     left: &'static str,
//     right: &'static str,
// ) -> impl FnMut(I) -> IResult<I, (Punctuator<'a>, Vec<T>, Recoverable<'a, Punctuator<'a>>), Error<'a>>
// where
//     T: Parse<'a>,
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     tuple((
//         punctuator_if(left),
//         many0(T::parse),
//         recoverable_punctuator_if(right),
//     ))
// }

// pub fn subgroup<'a, I>(input: I) -> IResult<I, Vec<Token<'a>>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     tuple((
//         punctuator_if("("),
//         many_till(Token::parse, peek(punctuator_if(")"))).map(|(tokens, _)| tokens),
//         punctuator_if(")"),
//     ))
//     .map(|(left, tokens, right)| {
//         once(Token::Punctuator(left))
//             .chain(tokens)
//             .chain(once(Token::Punctuator(right)))
//             .collect()
//     })
//     .parse(input)
// }

// pub fn recoverable_group<'a, T, I>(
//     left: &'static str,
//     right: &'static str,
// ) -> impl Parser<
//     I,
//     (
//         Punctuator<'a>,
//         Vec<Recoverable<'a, T>>,
//         Recoverable<'a, Punctuator<'a>>,
//     ),
//     Error<'a>,
// >
// where
//     T: Parse<'a> + Clone,
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     tuple((
//         punctuator_if(left),
//         recoverable_many(T::parse, || subgroup, move || lower_delimiter_or_eof(left)),
//         recoverable_punctuator_if(right),
//     ))
// }

// pub fn recoverable_tuple<'a, F1, F2, F3, F4, F5, O1, O2, O3, O4, O5, I>(
//     parsers: (F1, F2, F3, F4, F5),
// ) -> impl Parser<I, ((O1, O2, O3, O4, O5), Rest<'a>), Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     F1: Parser<I, O1, Error<'a>>,
//     F2: Parser<I, O2, Error<'a>>,
//     F3: Parser<I, O3, Error<'a>>,
//     F4: Parser<I, O4, Error<'a>>,
//     F5: Parser<I, O5, Error<'a>>,
// {
//     tuple((
//         parsers.0,
//         rest.and(parsers.1),
//         rest.and(parsers.2),
//         rest.and(parsers.3),
//         rest.and(parsers.4),
//     ))
//     .map(|(o1, (r2, o2), (r3, o3), (r4, o4), (r5, o5))| {
//         let rest = r2.into_iter().chain(r3).chain(r4).chain(r5).collect();
//         ((o1, o2, o3, o4, o5), rest)
//     })
// }

// pub fn recoverable_opt<'a, I, P, O, S, SO>(
//     parser: P,
//     sequence_point: S,
// ) -> impl Parser<I, Option<O>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
//     P: Parser<I, O, Error<'a>>,
//     S: Parser<I, SO, Error<'a>>,
// {
//     parser.map(Some).or(peek(sequence_point).map(|_| None))
// }

// pub fn rest<'a, I>(input: I) -> IResult<I, Rest<'a>, Error<'a>>
// where
//     I: Iterator<Item = Token<'a>> + Clone + InputLength,
// {
//     many_till(
//         Token::parse,
//         peek(
//             sequence_point
//                 .map(|_| ())
//                 .or(eof.map(|_| ()))
//                 .or(Name::parse.map(|_| ())),
//         ),
//     )
//     .map(|(tokens, _)| match tokens {
//         tokens if !tokens.is_empty() => Rest(vec![Err(tokens)]),
//         _ => Rest(vec![]),
//     })
//     .parse(input)
// }
