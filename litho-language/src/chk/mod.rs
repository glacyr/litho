use litho_diagnostics::Diagnostic;

use crate::ast::{Node, Recoverable, Visit};
use crate::lex::{Span, Token};

pub trait Errors<T> {
    fn errors(&self) -> Vec<Diagnostic<Span>>;
}

impl<T, N> Errors<T> for N
where
    N: Node<T>,
{
    fn errors(&self) -> Vec<Diagnostic<Span>> {
        let mut errors = vec![];
        self.traverse(&CollectErrors, &mut errors);
        errors
    }
}

pub fn collect_errors<N, T>(ast: &(N, Vec<Token<T>>)) -> Vec<Diagnostic<Span>>
where
    N: Node<T>,
    T: Clone,
{
    let mut errors = vec![];
    ast.0.traverse(&CollectErrors, &mut errors);
    match ast.1.first().zip(ast.1.last()) {
        Some((first, last)) => errors.push(Diagnostic::unrecognized_tokens(
            first.span().joined(last.span()),
        )),
        None => {}
    };
    errors
}

pub struct CollectErrors;

impl<'ast, T> Visit<'ast, T> for CollectErrors
where
    T: 'ast,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_recoverable<U>(
        &self,
        node: &'ast Recoverable<U>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node {
            Recoverable::Present(_) => {}
            Recoverable::Missing(error) => accumulator.push(error.to_diagnostic()),
        }
    }
}
