mod diagnostics;
mod error;

pub use diagnostics::{IntoReport, LabelBuilder, ReportBuilder};
pub use error::Error;

use crate::ast::{Node, Recoverable, Visit};
use crate::lex::Token;

pub trait Errors<T> {
    fn errors<'ast>(&'ast self) -> Vec<Error<'ast, T>>
    where
        T: 'ast;
}

impl<T, N> Errors<T> for N
where
    N: Node<T>,
{
    fn errors<'ast>(&'ast self) -> Vec<Error<'ast, T>>
    where
        T: 'ast,
    {
        let mut errors = vec![];
        self.traverse(&CollectErrors, &mut errors);
        errors
    }
}

pub fn collect_errors<N, T>(ast: &(N, Vec<Token<T>>)) -> Vec<Error<T>>
where
    N: Node<T>,
    T: Clone,
{
    let mut errors = vec![];
    ast.0.traverse(&CollectErrors, &mut errors);
    errors.extend(ast.1.iter().map(|token| Error::UnrecognizedTokens {
        tokens: vec![token.clone()],
    }));
    errors
}

pub struct CollectErrors;

impl<'ast, T> Visit<'ast, T> for CollectErrors
where
    T: 'ast,
{
    type Accumulator = Vec<Error<'ast, T>>;

    fn visit_recoverable<U>(
        &self,
        node: &'ast Recoverable<U>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node {
            Recoverable::Present(_) => {}
            Recoverable::Missing(error) => accumulator.push(Error::Recoverable(error)),
        }
    }
}
