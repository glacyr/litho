use crate::lex::{Name, Punctuator};

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
