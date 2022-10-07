// #![warn(missing_docs)]

pub mod ast;
pub mod chk;
pub mod lex;
pub mod syn;

pub use ast::Document;
pub use syn::{parse_from_str, Parse};

#[doc(hidden)]
pub use ariadne;

#[cfg(test)]
mod tests {
    use ariadne::Source;

    use super::chk::{Error, Errors, IntoReport};
    use super::lex::{Span, Token};
    use super::{Document, Parse};

    #[test]
    fn it_works() {
        let source = r#"scalar Example @specifiedBy(url: "https://example.com")"#;
        let (ast, unrecognized) = Document::parse_from_str(0, source).unwrap();

        println!("Result: {:#?} (errors: {:#?})", ast, ast.errors());

        for error in
            ast.errors()
                .into_iter()
                .chain(unrecognized.into_iter().map(|token: Token<&str>| {
                    Error::UnrecognizedTokens {
                        tokens: vec![token],
                    }
                }))
        {
            error
                .into_report::<ariadne::ReportBuilder<Span>>()
                .print((0usize, Source::from(source)))
                .unwrap();
        }
    }
}
