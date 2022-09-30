use std::collections::VecDeque;

use logos::Logos;
use nom::InputLength;

use super::raw::{raw_lexer, RawLexer, RawToken};
use super::{Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct Error<'a>(RawToken<'a>);

impl<'a> Error<'a> {
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
pub struct Name<'a>(RawToken<'a>);

impl<'a> Name<'a> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

impl<'a> AsRef<str> for Name<'a> {
    fn as_ref(&self) -> &str {
        &self.0.source
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
pub struct Punctuator<'a>(RawToken<'a>);

impl<'a> Punctuator<'a> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

impl<'a> AsRef<str> for Punctuator<'a> {
    fn as_ref(&self) -> &str {
        &self.0.source
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Token<'a> {
    Error(Error<'a>),

    /// Represents a [Name] in GraphQL.
    Name(Name<'a>),

    /// Represents a [Punctuator] in GraphQL.
    Punctuator(Punctuator<'a>),
}

impl<'a> Token<'a> {
    pub fn span(&self) -> Span {
        match self {
            Token::Error(token) => token.0.span,
            Token::Name(token) => token.0.span,
            Token::Punctuator(token) => token.0.span,
        }
    }
}

impl<'a> From<RawToken<'a>> for Token<'a> {
    fn from(raw: RawToken<'a>) -> Self {
        match raw.kind {
            TokenKind::Error => Token::Error(Error(raw)),
            TokenKind::Name => Token::Name(Name(raw)),
            TokenKind::Punctuator => Token::Punctuator(Punctuator(raw)),
            _ => unreachable!("All other token types should have been ignored by the lexer."),
        }
    }
}

#[derive(Clone)]
pub struct Lexer<'a> {
    lexer: RawLexer<'a>,
}

impl<'a> Lexer<'a> {
    pub fn exact(self) -> ExactLexer<'a> {
        ExactLexer {
            tokens: self.collect(),
            last_span: None,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.lexer.next()?.into())
    }
}

#[derive(Clone)]
pub struct ExactLexer<'a> {
    tokens: VecDeque<Token<'a>>,
    last_span: Option<Span>,
}

impl<'a> ExactLexer<'a> {
    pub fn span(&self) -> Span {
        match (self.last_span.as_ref(), self.tokens.get(0)) {
            (Some(&left), Some(right)) => Span::between(left, right.span()),
            (Some(&left), None) => left.collapse_to_end(),
            (None, Some(right)) => right.span().collapse_to_start(),
            (None, None) => todo!(),
        }
    }
}

impl<'a> Iterator for ExactLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.pop_front();
        self.last_span = token.as_ref().map(|token| token.span());
        token
    }
}

impl<'a> InputLength for ExactLexer<'a> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

pub fn lexer(source_id: usize, source: &str) -> Lexer {
    Lexer {
        lexer: raw_lexer(source_id, TokenKind::lexer(source)),
    }
}
