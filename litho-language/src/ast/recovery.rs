use std::iter::FromIterator;

use crate::lex::Token;

use super::{Node, Visit};

pub type Recoverable<'a, T> = Result<T, Vec<Token<'a>>>;

impl<'a, T> Node<'a> for Recoverable<'a, T>
where
    T: Node<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        visitor.visit_recoverable(self, accumulator);

        match self {
            Ok(value) => value.traverse(visitor, accumulator),
            Err(errors) => errors
                .iter()
                .for_each(|error| visitor.visit_span(error.span(), accumulator)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rest<'a>(pub Vec<Recoverable<'a, ()>>);

impl<'a> From<Recoverable<'a, ()>> for Rest<'a> {
    fn from(recoverable: Recoverable<'a, ()>) -> Self {
        Rest(vec![recoverable])
    }
}

impl<'a> IntoIterator for Rest<'a> {
    type Item = Recoverable<'a, ()>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> FromIterator<Recoverable<'a, ()>> for Rest<'a> {
    fn from_iter<T: IntoIterator<Item = Recoverable<'a, ()>>>(iter: T) -> Self {
        Rest(iter.into_iter().collect())
    }
}

impl<'a> Node<'a> for Rest<'a> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        self.0
            .iter()
            .for_each(|recoverable| recoverable.traverse(visitor, accumulator))
    }
}
