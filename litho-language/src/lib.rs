mod ast;
mod chk;
mod lex;
mod syn;

pub use ast::*;
pub use chk::{Errors, IntoReport, LabelBuilder, ReportBuilder};
pub use lex::{Name, Punctuator, Span, Token, TokenKind};
pub use syn::Parse;

pub use ariadne;

#[cfg(test)]
mod tests {
    use ariadne::Source;

    use super::chk::{Errors, IntoReport};
    use super::lex::Span;
    use super::syn::Parse;
    use super::*;

    #[test]
    fn it_works() {
        let source = r#"query Example
        "#;
        let ast = Document::parse_from_str(0, source).unwrap();

        println!("Result: {:#?} (errors: {:#?})", ast, ast.errors());

        for error in ast.errors() {
            error
                .into_report::<ariadne::ReportBuilder<Span>>()
                .print((0usize, Source::from(source)))
                .unwrap();
        }
    }
}
