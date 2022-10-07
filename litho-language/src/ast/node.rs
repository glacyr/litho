use crate::lex::Span;

use super::{Recoverable, Visit};

pub trait Node<T> {
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>;

    fn span(&self) -> Span {
        let mut span = None;
        self.traverse(&SpanCollector, &mut span);
        span.unwrap_or_default()
    }
}

pub struct SpanCollector;

impl<'ast, T> Visit<'ast, T> for SpanCollector {
    type Accumulator = Option<Span>;

    fn visit_span(&self, span: crate::lex::Span, accumulator: &mut Self::Accumulator) {
        accumulator.get_or_insert(span).join(span)
    }
}

impl<T, N> Node<T> for Vec<N>
where
    N: Node<T>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        self.iter()
            .for_each(|item| item.traverse(visitor, accumulator))
    }
}

impl<T, N> Node<T> for Option<N>
where
    N: Node<T>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        self.iter()
            .for_each(|item| item.traverse(visitor, accumulator))
    }
}

impl<T, A, B> Node<T> for (A, B)
where
    A: Node<T>,
    B: Node<T>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        let (a, b) = self;
        a.traverse(visitor, accumulator);
        b.traverse(visitor, accumulator);
    }
}

impl<T, N> Node<T> for Recoverable<N>
where
    N: Node<T>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        visitor.visit_recoverable(self, accumulator);

        match self {
            Recoverable::Present(value) => value.traverse(visitor, accumulator),
            Recoverable::Missing(_) => {}
        }
    }
}

macro_rules! node {
    ($ty:ident, $visit:ident, $($fields:ident),*) => {
        impl<T> Node<T> for $ty<T> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, T>,
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
        impl<T> Node<T> for $ty<T> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, T>,
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
        impl<T> Node<T> for $ty<T> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, T>,
            {
                visitor.$visit(self, accumulator);

                self.0.traverse(visitor, accumulator);
            }
        }
    };
}

pub(crate) use node_unit;
