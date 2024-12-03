use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue};

use super::{ContextValue, Node, Visit};

impl<'a, T> Node<'a, T> for Name<'a, T>
where
    T: ContextValue<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a, T>,
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

impl<'a, T> Node<'a, T> for Punctuator<'a, T>
where
    T: ContextValue<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a, T>,
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

impl<'a, T> Node<'a, T> for IntValue<'a, T>
where
    T: ContextValue<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a, T>,
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

impl<'a, T> Node<'a, T> for FloatValue<'a, T>
where
    T: ContextValue<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a, T>,
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

impl<'a, T> Node<'a, T> for StringValue<'a, T>
where
    T: ContextValue<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a, T>,
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
