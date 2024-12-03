use litho_diagnostics::Diagnostic;

use crate::ast::{
    ContextValue, Node, Recoverable, TypeSystemDefinition, TypeSystemExtension, Visit,
};
use crate::lex::{Span, Token};

pub trait Errors<T> {
    fn errors(&self) -> Vec<Diagnostic<Span>>;
}

impl<'a, T, N> Errors<T> for N
where
    T: ContextValue<'a>,
    N: Node<'a, T>,
{
    fn errors(&self) -> Vec<Diagnostic<Span>> {
        let mut errors = vec![];
        self.traverse(&CollectErrors, &mut errors);
        errors
    }
}

pub fn collect_errors<'a, N, T>(ast: &(N, Vec<Token<'a, T>>)) -> Vec<Diagnostic<Span>>
where
    T: ContextValue<'a>,
    N: Node<'a, T>,
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

impl<'ast, 'a, T> Visit<'ast, 'a, T> for CollectErrors
where
    T: ContextValue<'a>,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_type_system_definition(
        &self,
        node: &'ast TypeSystemDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let TypeSystemDefinition::Error(error) = node {
            accumulator.push(Diagnostic::unrecognized_tokens(error.span()))
        }
    }

    fn visit_type_system_extension(
        &self,
        node: &'ast TypeSystemExtension<'a, T>,
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
