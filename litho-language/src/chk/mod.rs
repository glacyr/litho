use litho_diagnostics::Diagnostic;

use crate::ast::{Node, Recoverable, TypeSystemDefinition, TypeSystemExtension, Visit};
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

    errors.extend(
        ast.1
            .iter()
            .map(Token::span)
            .map(Diagnostic::unrecognized_tokens),
    );
    errors
}

pub struct CollectErrors;

impl<'ast, T> Visit<'ast, T> for CollectErrors
where
    T: 'ast,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_type_system_definition(
        &self,
        node: &'ast TypeSystemDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let TypeSystemDefinition::Error(error) = node {
            accumulator.push(Diagnostic::unrecognized_tokens(error.span()))
        }
    }

    fn visit_type_system_extension(
        &self,
        node: &'ast TypeSystemExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let TypeSystemExtension::Error(error) = node {
            accumulator.push(Diagnostic::unrecognized_tokens(error.span()))
        }
    }

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
