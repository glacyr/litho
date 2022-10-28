use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue};

use super::{Node, Visit};

impl<T> Node<T> for Name<T> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        visitor.visit_span(self.span(), accumulator);
    }

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.as_raw_token().congruent(other.as_raw_token())
    }
}

impl<T> Node<T> for Punctuator<T> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        visitor.visit_span(self.span(), accumulator);
    }

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.as_raw_token().congruent(other.as_raw_token())
    }
}

impl<T> Node<T> for IntValue<T> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        visitor.visit_int_value(self, accumulator);
        visitor.visit_span(self.span(), accumulator);
    }

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.as_raw_token().congruent(other.as_raw_token())
    }
}

impl<T> Node<T> for FloatValue<T> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        visitor.visit_float_value(self, accumulator);
        visitor.visit_span(self.span(), accumulator);
    }

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.as_raw_token().congruent(other.as_raw_token())
    }
}

impl<T> Node<T> for StringValue<T> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        visitor.visit_string_value(self, accumulator);
        visitor.visit_span(self.span(), accumulator);
    }

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.as_raw_token().congruent(other.as_raw_token())
    }
}
