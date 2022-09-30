use std::collections::VecDeque;

use logos::Logos;
use nom::InputLength;

use super::{raw_lexer, RawLexer, RawToken, Span, TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct Error<'a>(RawToken<'a>);

impl<'a> Error<'a> {
    pub fn span(&self) -> Span {
        self.0.span
    }
}

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
    Name(Name<'a>),
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
        let raw = self.lexer.next()?;

        Some(match raw.kind {
            TokenKind::Error => Token::Error(Error(raw)),
            TokenKind::Name => Token::Name(Name(raw)),
            TokenKind::Punctuator => Token::Punctuator(Punctuator(raw)),
            _ => unreachable!("Raw: {:#?}", raw),
        })
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
