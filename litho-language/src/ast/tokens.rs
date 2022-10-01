use crate::lex::{FloatValue, IntValue, Name, Punctuator, StringValue};

use super::{Node, Visit};

impl<'a> Node<'a> for Name<'a> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        visitor.visit_span(self.span(), accumulator);
    }
}

impl<'a> Node<'a> for Punctuator<'a> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        visitor.visit_span(self.span(), accumulator);
    }
}

impl<'a> Node<'a> for IntValue<'a> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        visitor.visit_int_value(self, accumulator);
        visitor.visit_span(self.span(), accumulator);
    }
}

impl<'a> Node<'a> for FloatValue<'a> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        visitor.visit_float_value(self, accumulator);
        visitor.visit_span(self.span(), accumulator);
    }
}

impl<'a> Node<'a> for StringValue<'a> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        visitor.visit_string_value(self, accumulator);
        visitor.visit_span(self.span(), accumulator);
    }
}
