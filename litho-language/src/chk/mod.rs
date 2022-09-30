mod diagnostics;
mod error;

pub use diagnostics::{IntoReport, LabelBuilder, ReportBuilder};
pub use error::Error;

use crate::ast::{Node, Recoverable, Visit};

pub trait Errors<'a> {
    fn errors<'ast>(&'ast self) -> Vec<Error<'ast, 'a>>
    where
        'a: 'ast;
}

impl<'a, T> Errors<'a> for T
where
    T: Node<'a>,
{
    fn errors<'ast>(&'ast self) -> Vec<Error<'ast, 'a>>
    where
        'a: 'ast,
    {
        let mut errors = vec![];
        self.traverse(&CollectErrors, &mut errors);
        errors
    }
}

pub struct CollectErrors;

impl<'ast, 'a> Visit<'ast, 'a> for CollectErrors
where
    'a: 'ast,
{
    type Accumulator = Vec<Error<'ast, 'a>>;

    fn visit_recoverable<T>(
        &self,
        node: &'ast Recoverable<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node {
            Recoverable::Present(_) => {}
            Recoverable::Missing(error) => accumulator.push(Error::Recoverable(error)),
        }
    }
}
