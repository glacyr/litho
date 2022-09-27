use crate::lex::Span;

use super::Visit;

pub trait Node<'a> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>;

    fn span(&self) -> Span {
        let mut span = None;
        self.traverse(&SpanCollector, &mut span);
        span.unwrap()
    }
}

pub struct SpanCollector;

impl<'ast, 'a> Visit<'ast, 'a> for SpanCollector {
    type Accumulator = Option<Span>;

    fn visit_span(&self, span: crate::lex::Span, accumulator: &mut Self::Accumulator) {
        accumulator.get_or_insert(span).join(span)
    }
}

impl<'a> Node<'a> for () {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
    }
}

impl<'a, T> Node<'a> for Vec<T>
where
    T: Node<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        self.iter()
            .for_each(|item| item.traverse(visitor, accumulator))
    }
}

impl<'a, T> Node<'a> for Option<T>
where
    T: Node<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        self.iter()
            .for_each(|item| item.traverse(visitor, accumulator))
    }
}

impl<'a, A, B> Node<'a> for (A, B)
where
    A: Node<'a>,
    B: Node<'a>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, 'a>,
    {
        let (a, b) = self;
        a.traverse(visitor, accumulator);
        b.traverse(visitor, accumulator);
    }
}

macro_rules! node {
    ($ty:ident, $visit:ident, $($fields:ident),*) => {
        impl<'a> Node<'a> for $ty<'a> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, 'a>,
            {
                visitor.$visit(self, accumulator);

                $(
                    self.$fields.traverse(visitor, accumulator);
                )*
            }
        }
    };
}

pub(crate) use node;

macro_rules! node_enum {
    ($ty:ident, $visit:ident, $($variants:ident),* $(,)?) => {
        impl<'a> Node<'a> for $ty<'a> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, 'a>,
            {
                visitor.$visit(self, accumulator);

                match self {
                    $(
                        Self::$variants(node) => node.traverse(visitor, accumulator),
                    )*
                }
            }
        }
    };
}

pub(crate) use node_enum;

macro_rules! node_unit {
    ($ty:ident, $visit:ident) => {
        impl<'a> Node<'a> for $ty<'a> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, 'a>,
            {
                visitor.$visit(self, accumulator);

                self.0.traverse(visitor, accumulator);
            }
        }
    };
}

pub(crate) use node_unit;
