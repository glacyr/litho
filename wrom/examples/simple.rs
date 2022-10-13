use nom::combinator::eof;
use nom::error::ParseError;
use nom::InputLength;

use wrom::{terminal, Input, Recoverable, RecoverableParser};

#[derive(Debug)]
pub struct Error;

impl<I> ParseError<I> for Error {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self
    }

    fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    A1,
    A2,
    B1,
    X,
    Y,
    C2,
}

pub fn token<I>(token: Token) -> impl RecoverableParser<I, Token, Error>
where
    I: Input<Item = Token>,
{
    terminal(move |mut input: I| match input.next() {
        Some(next) if next == token => Ok((input, next)),
        Some(_) | None => Err(nom::Err::Error(Error)),
    })
}

#[derive(Debug, Default)]
pub struct Missing(&'static str);

impl wrom::Missing for Missing {
    type Error = &'static str;
}

#[derive(Debug)]
pub struct Root {
    a: A,
    b: Recoverable<B, Missing>,
    c: Recoverable<C, Missing>,
}

#[derive(Debug)]
pub struct A {
    a1: Token,
    a2: Recoverable<Token, Missing>,
}

pub fn a<I>() -> impl RecoverableParser<I, A, Error>
where
    I: Input<Item = Token, Missing = Missing>,
{
    token(Token::A1)
        .and_recover(token(Token::A2), |_| Missing("missing A2"))
        .map(|(a1, a2)| A { a1, a2 })
}

#[derive(Debug)]
pub struct B {
    b1: Token,
}

pub fn b<I>() -> impl RecoverableParser<I, B, Error>
where
    I: Input<Item = Token>,
{
    token(Token::B1).map(|b1| B { b1 })
}

#[derive(Debug)]
pub struct C {
    c1: C1,
    c2: Recoverable<Token, Missing>,
}

pub fn c<I>() -> impl RecoverableParser<I, C, Error>
where
    I: Input<Item = Token, Missing = Missing>,
{
    c1().and_recover(token(Token::C2), |_| Missing("missing C2"))
        .map(|(c1, c2)| C { c1, c2 })
}
#[derive(Debug)]
pub struct C1 {
    x: Token,
    y: Recoverable<Token, Missing>,
}

pub fn c1<I>() -> impl RecoverableParser<I, C1, Error>
where
    I: Input<Item = Token, Missing = Missing>,
{
    token(Token::X)
        .and_recover(token(Token::Y), |_| Missing("missing Y"))
        .map(|(x, y)| C1 { x, y })
}

pub fn root<I>() -> impl RecoverableParser<I, Root, Error>
where
    I: Input<Item = Token, Missing = Missing>,
{
    a().and_recover(b(), |_| Missing("missing B"))
        .and_recover(c(), |_| Missing("missing C"))
        .map(|((a, b), c)| Root { a, b, c })
}

#[derive(Clone, Debug)]
pub struct Stream<I>(I);

impl<I> InputLength for Stream<I>
where
    I: ExactSizeIterator,
{
    fn input_len(&self) -> usize {
        self.0.len()
    }
}

impl<I> Iterator for Stream<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<A, I> Extend<A> for Stream<I> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        //
    }
}

impl<I> Input for Stream<I>
where
    I: Clone + ExactSizeIterator,
{
    type Item = I::Item;
    type Missing = Missing;

    fn missing(&self, missing: Missing) -> &'static str {
        missing.0
    }
}

fn main() {
    // fn parse_a<I, E>(input: I) -> IResult<I, A, Error>
    // where
    //     I: Iterator<Item = Token> + Clone + InputLength,
    //     E: ParseError<I>,
    // {
    //     a::<I, _>(eof).parse(input)
    // }

    let input = vec![
        Token::A1,
        Token::Y,
        Token::X,
        Token::A1,
        Token::A2,
        Token::C2,
    ];
    eprintln!(
        "Output: {:#?}",
        root().parse(Stream(input.into_iter()), terminal(eof))
    );
}
