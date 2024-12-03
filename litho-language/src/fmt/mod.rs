mod common;
mod diff;
mod executable;
mod macros;
mod measurer;
mod schema;
mod tokens;
mod types;

pub use diff::Diff;
pub use measurer::Measurer;
pub use types::{Format, Formatter, Shape};

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    use super::Format;

    use crate::ast::Document;
    use crate::syn::Parse;

    #[test]
    fn test_example() {
        let bump = Bump::new();

        let node: Document<&str> = Document::parse_from_str(
            Default::default(),
            "\"\"\"
            Hello World!
            \"\"\"
            schema @litho(url: [{ hello: \"world\", blabla: true }, true]) {
            query: HelloQuery
            mutation: HelloMutation
        }

        \"\"\"
        Long description
            Some additional
       This is weird
       \"\"\"
        type Example

        query example { id }
        ",
            &bump,
        )
        .unwrap()
        .0;

        eprintln!("Output: {}", node.format_to_string(80));
    }
}
