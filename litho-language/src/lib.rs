mod ast;
mod chk;
mod lex;
pub mod syn;

pub use ast::*;
pub use chk::{Errors, IntoReport, LabelBuilder, ReportBuilder};
pub use lex::{Name, Punctuator, Span, Token, TokenKind};
// pub use syn::Parse;

pub use ariadne;

#[cfg(test)]
mod tests {
    use ariadne::Source;

    use super::chk::{Error, Errors, IntoReport};
    use super::lex::Span;
    use super::syn;
    use super::*;

    #[test]
    fn it_works() {
        let source = r#"query Example($example: Hello, $world: Int) {
            query- {
                hello(x-: }

                    fragment Example {
                        value
                    }
        "#;
        let (unrecognized, ast) = syn::parse_from_str(syn::document(), 0, source).unwrap();

        println!("Result: {:#?} (errors: {:#?})", ast, ast.errors());

        for error in ast
            .errors()
            .into_iter()
            .chain(
                unrecognized
                    .into_iter()
                    .map(|token| Error::UnrecognizedTokens {
                        tokens: vec![token],
                    }),
            )
        {
            error
                .into_report::<ariadne::ReportBuilder<Span>>()
                .print((0usize, Source::from(source)))
                .unwrap();
        }
    }
}
