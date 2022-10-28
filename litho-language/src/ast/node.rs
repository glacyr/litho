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

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq;
}

impl<N, T> Node<T> for &N
where
    N: Node<T>,
{
    fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
    where
        V: Visit<'ast, T>,
    {
        (*self).traverse(visitor, accumulator)
    }

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        (*self).congruent(*other)
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

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        if self.len() == other.len() {
            self.iter().zip(other.iter()).all(|(a, b)| a.congruent(b))
        } else {
            false
        }
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

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        match (self, other) {
            (Some(lhs), Some(rhs)) => lhs.congruent(rhs),
            (None, None) => true,
            (_, _) => false,
        }
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

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.0.congruent(&other.0) && self.1.congruent(&other.1)
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

    fn congruent(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        self.ok().congruent(&other.ok())
    }
}

macro_rules! node {
    (Arc<$ty:ident>, $visit:ident $(+ $post:ident)?, $($fields:ident),*) => {
        node!(Arc<$ty<T>>, $visit $(+ $post)?, $($fields),*);
    };
    ($ty:ident, $visit:ident $(+ $post:ident)?, $($fields:ident),*) => {
        node!($ty<T>, $visit $(+ $post)?, $($fields),*);
    };
    ($ty:ty, $visit:ident $(+ $post:ident)?, $($fields:ident),*) => {
        impl<T> Node<T> for $ty {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, T>,
            {
                visitor.$visit(self, accumulator);

                $(
                    self.$fields.traverse(visitor, accumulator);
                )*

                $(visitor.$post(self, accumulator);)?
            }

            fn congruent(&self, other: &Self) -> bool
            where
                T: PartialEq,
            {
                $(
                    self.$fields.congruent(&other.$fields) &&
                )*
                true
            }
        }
    };
}

pub(crate) use node;

macro_rules! node_enum {
    (Arc<$ty:ident>, $visit:ident $(+ $post:ident)?, $($variants:ident),* $(,)?) => {
        impl<T> Node<T> for $ty<T> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, T>,
            {
                match self {
                    $(
                        Self::$variants(node) => node.traverse(visitor, accumulator),
                    )*
                }
            }

            fn congruent(&self, other: &Self) -> bool
            where
                T: PartialEq,
            {
                match (self, other) {
                    $(
                        (Self::$variants(lhs), Self::$variants(rhs)) => lhs.congruent(rhs),
                    )*
                    (_, _) => false,
                }
            }
        }

        impl<T> Node<T> for Arc<$ty<T>> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, T>,
            {
                visitor.$visit(self, accumulator);

                self.as_ref().traverse(visitor, accumulator);
            }

            fn congruent(&self, other: &Self) -> bool
            where
                T: PartialEq,
            {
                self.as_ref().congruent(other.as_ref())
            }
        }
    };
    ($ty:ident, $visit:ident $(+ $post:ident)?, $($variants:ident),* $(,)?) => {
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

                $(visitor.$post(self, accumulator);)?
            }

            fn congruent(&self, other: &Self) -> bool
            where
                T: PartialEq,
            {
                match (self, other) {
                    $(
                        (Self::$variants(lhs), Self::$variants(rhs)) => lhs.congruent(rhs),
                    )*
                    (_, _) => false,
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

            fn congruent(&self, other: &Self) -> bool
            where
                T: PartialEq,
            {
                self.0.congruent(&other.0)
            }
        }
    };
}

pub(crate) use node_unit;

macro_rules! node_arc {
    ($ty:ident) => {
        impl<T> Node<T> for Arc<$ty<T>> {
            fn traverse<'ast, V>(&'ast self, visitor: &V, accumulator: &mut V::Accumulator)
            where
                V: Visit<'ast, T>,
            {
                self.as_ref().traverse(visitor, accumulator);
            }

            fn congruent(&self, other: &Self) -> bool
            where
                T: PartialEq,
            {
                self.as_ref().congruent(other.as_ref())
            }
        }
    };
}

pub(crate) use node_arc;
