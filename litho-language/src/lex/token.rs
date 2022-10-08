use std::borrow::Borrow;
use std::collections::VecDeque;

use logos::Logos;
use nom::InputLength;
use unindent::unindent;

use super::raw::{raw_lexer, RawLexer, RawToken};
use super::{Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct Error<T>(RawToken<T>);

impl<T> Error<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

/// Represents a name in a GraphQL document.
///
/// ```bnf
/// Name ::= NameStart NameContinue*
///
/// NameStart ::= Letter
///
/// NameContinue ::= Letter | Digit
///
/// Letter ::= A | B | C | D | E | F | G | H | I | J | K | L | M |
///            N | O | P | Q | R | S | T | U | V | W | X | Y | Z |
///            a | b | c | d | e | f | g | h | i | j | k | l | m |
///            n | o | p | q | r | s | t | u | v | w | x | y | z
///
/// Digit ::= 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
/// ```
///
/// GraphQL Documents are full of named things: operations, fields, arguments,
/// types directives, fragments, and variables. All names must follow the same
/// grammatical form.
///
/// Names in GraphQL are case-sensitive. That is to say `name`, `Name` and
/// `NAME` all refer to different names.
///
/// A _Name_ must not be followed by a _NameContinue_. In other words, a _Name_
/// token is always the longest possible valid sequence. The source characters
/// `a1` cannot be interpreted as two tokens since `a` is followed by the
/// _NameContinue_ `1`.
///
/// Note: Names in GraphQL are limited to the Latin ASCII subset of
/// _SourceCharacter_ in order to support interoperation with as many other
/// systems as possible.
///
/// ##### Reserved Names
///
/// Any _Name_ within a GraphQL type system must not start with two underscores
/// "__" unless it is part of the introspection system as defined by this
/// specification.
///
/// _Source: [Sec. 2.1.9 Names](https://spec.graphql.org/October2021/#sec-Names)_
#[derive(Clone, Copy, Debug)]
pub struct Name<T>(RawToken<T>);

impl<T> Name<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

impl<T> AsRef<T> for Name<T> {
    fn as_ref(&self) -> &T {
        self.0.source.borrow()
    }
}

/// Represents a punctuator in a GraphQL document.
///
/// ```bnf
/// Punctuator ::= ! | $ | & | ( | ) | ... | : | = | @ | [ | ] | { | "|" | }
/// ```
///
/// GraphQL documents include punctuation in order to describe structure.
/// GraphQL is a data description language and not a programming language,
/// therefore GraphQL lacks the punctuation often used to describe mathematical
/// expressions.
///
/// _Source: [Sec: 2.1.8 Punctuators](https://spec.graphql.org/October2021/#sec-Punctuators)_
///
/// __Implementation note:__ any punctuator that's not part of the grammar
/// listed above is considered an [Error].
#[derive(Clone, Copy, Debug)]
pub struct Punctuator<T>(RawToken<T>);

impl<T> Punctuator<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

impl<T> AsRef<T> for Punctuator<T> {
    fn as_ref(&self) -> &T {
        self.0.source.borrow()
    }
}

/// Represents an int value (literal) in a GraphQL document.
#[derive(Clone, Copy, Debug)]
pub struct IntValue<T>(RawToken<T>);

impl<T> IntValue<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

/// Represents a float value (literal) in a GraphQL document.
#[derive(Clone, Copy, Debug)]
pub struct FloatValue<T>(RawToken<T>);

impl<T> FloatValue<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StringValue<T>(RawToken<T>);

impl<T> StringValue<T> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

impl<T> ToString for StringValue<T>
where
    T: ToString,
{
    fn to_string(&self) -> String {
        let source = self.0.source.to_string();

        match source.starts_with("\"\"\"") {
            true => unindent(&source[3..source.len() - 3]),
            false => source[1..source.len() - 1].replace("\\\"", "\""),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Token<T> {
    Error(Error<T>),

    /// Represents a [Name] in GraphQL.
    Name(Name<T>),

    /// Represents a [Punctuator] in GraphQL.
    Punctuator(Punctuator<T>),

    /// Represents an [IntValue] in GraphQL.
    IntValue(IntValue<T>),

    /// Represents a [FloatValue] in GraphQL.
    FloatValue(FloatValue<T>),

    /// Represents a [StringValue] in GraphQL.
    StringValue(StringValue<T>),
}

impl<T> Token<T> {
    pub fn span(&self) -> Span {
        match self {
            Token::Error(token) => token.0.span,
            Token::Name(token) => token.0.span,
            Token::Punctuator(token) => token.0.span,
            Token::IntValue(token) => token.0.span,
            Token::FloatValue(token) => token.0.span,
            Token::StringValue(token) => token.0.span,
        }
    }
}

impl<T> From<RawToken<T>> for Token<T> {
    fn from(raw: RawToken<T>) -> Self {
        match raw.kind {
            TokenKind::Error => Token::Error(Error(raw)),
            TokenKind::Name => Token::Name(Name(raw)),
            TokenKind::Punctuator => Token::Punctuator(Punctuator(raw)),
            TokenKind::IntValue => Token::IntValue(IntValue(raw)),
            TokenKind::FloatValue => Token::FloatValue(FloatValue(raw)),
            TokenKind::StringValue => Token::StringValue(StringValue(raw)),
            _ => unreachable!("All other token types should have been ignored by the lexer."),
        }
    }
}

#[derive(Clone)]
pub struct Lexer<'a, T> {
    lexer: RawLexer<'a, T>,
}

impl<'a, T> Lexer<'a, T>
where
    T: From<&'a str>,
{
    pub fn exact(self) -> ExactLexer<T> {
        ExactLexer {
            tokens: self.collect(),
            last_span: None,
        }
    }
}

impl<'a, T> Iterator for Lexer<'a, T>
where
    T: From<&'a str>,
{
    type Item = Token<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.lexer.next()?.into())
    }
}

#[derive(Clone)]
pub struct FastLexer<'a, T> {
    tokens: &'a [Token<T>],
    position: usize,
    last_span: Option<Span>,
}

impl<'a, T> FastLexer<'a, T> {
    pub fn new(tokens: &'a [Token<T>]) -> FastLexer<'a, T> {
        FastLexer {
            tokens,
            position: 0,
            last_span: Default::default(),
        }
    }

    pub fn span(&self) -> Span {
        match (self.last_span.as_ref(), self.tokens.get(self.position)) {
            (Some(&left), Some(right)) => Span::between(left, right.span()),
            (Some(&left), None) => left.collapse_to_end(),
            (None, Some(right)) => right.span().collapse_to_start(),
            (None, None) => todo!(),
        }
    }
}

impl<'a, T> Iterator for FastLexer<'a, T>
where
    T: Clone,
{
    type Item = Token<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.get(self.position) {
            Some(token) => {
                self.position += 1;
                self.last_span.replace(token.span());
                Some(token.clone())
            }
            None => None,
        }
    }
}

impl<'a, T> InputLength for FastLexer<'a, T> {
    fn input_len(&self) -> usize {
        self.tokens.len() - self.position
    }
}

#[derive(Clone)]
pub struct ExactLexer<T> {
    pub tokens: VecDeque<Token<T>>,
    last_span: Option<Span>,
}

impl<T> ExactLexer<T> {
    pub fn span(&self) -> Span {
        match (self.last_span.as_ref(), self.tokens.get(0)) {
            (Some(&left), Some(right)) => Span::between(left, right.span()),
            (Some(&left), None) => left.collapse_to_end(),
            (None, Some(right)) => right.span().collapse_to_start(),
            (None, None) => todo!(),
        }
    }
}

impl<T> Iterator for ExactLexer<T> {
    type Item = Token<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.pop_front();
        self.last_span = token.as_ref().map(|token| token.span());
        token
    }
}

impl<T> InputLength for ExactLexer<T> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

pub fn lexer<T>(source_id: usize, source: &str) -> Lexer<T> {
    let _: <TokenKind as Logos>::Source;

    Lexer {
        lexer: raw_lexer(source_id, TokenKind::lexer(source)),
    }
}

mod display {
    use std::fmt::{Display, Formatter, Result};

    use super::Name;

    impl<T> Display for Name<T>
    where
        T: Display,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.0.source.fmt(f)
        }
    }
}
