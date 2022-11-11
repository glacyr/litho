mod common;
mod executable;
mod macros;
mod schema;
mod tokens;
mod types;

pub use types::{Format, Formatter};

#[cfg(test)]
mod tests {
    use super::Format;

    use crate::ast::Document;
    use crate::syn::Parse;

    #[test]
    fn test_example() {
        let node: Document<&'static str> = Document::parse_from_str(
            Default::default(),
            "\"\"\"
            Hello World!
            \"\"\"
            schema @litho(url: [{ hello: \"world\", blabla: true }, true]) {
            query: HelloQuery
            mutation: HelloMutation
        }
        
        type Example {
            complex(\"Abc\" nested: Boolean! = false, \"Hello\" example: Int!): Hello!
        }
        ",
        )
        .unwrap()
        .0;

        eprintln!("Output: {}", node.format_to_string(80));
    }
}
