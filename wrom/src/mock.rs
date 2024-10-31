use nom::error::{ErrorKind, ParseError};
use nom::InputLength;

use super::{terminal, Input, RecoverableParser};

#[derive(Clone)]
pub struct CollectUnrecognized<I>
where
    I: Input,
{
    input: I,
    unrecognized: Vec<I::Item>,
}

impl<I> CollectUnrecognized<I>
where
    I: Input,
{
    pub fn new(input: I) -> CollectUnrecognized<I> {
        CollectUnrecognized {
            input,
            unrecognized: Default::default(),
        }
    }

    pub fn into_inner(self) -> I {
        self.input
    }

    pub fn unrecognized(&mut self) -> impl Iterator<Item = I::Item> + '_ {
        self.unrecognized.drain(..)
    }
}

impl<I> InputLength for CollectUnrecognized<I>
where
    I: Input,
{
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<I> Input for CollectUnrecognized<I>
where
    I: Input + Clone,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next()
    }

    fn unrecognized<I2>(&mut self, iter: I2)
    where
        I2: IntoIterator<Item = Self::Item>,
    {
        self.unrecognized.extend(iter);
    }
}

pub fn char<I, E>(c: char) -> impl RecoverableParser<I, char, E>
where
    I: Input<Item = char> + Clone,
    E: ParseError<I>,
{
    terminal(move |mut input: I| match input.next() {
        Some(v) if c == v => Ok((input, v)),
        _ => Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Char))),
    })
}
