use graphql_parser::query::Text;
use graphql_parser::{query, schema};

pub mod diagnostics;
mod error;
mod extensions;
mod validators;
mod walk;

pub use error::Error;
pub use walk::{Scope, Traverse, Visitor};

pub fn validate<'a, 'b, T>(
    schema: &'a schema::Document<'b, T>,
    query: &'a query::Document<'b, T>,
) -> Result<(), Vec<Error<'a, 'b, T>>>
where
    'a: 'b,
    T: Text<'a>,
{
    let mut errors = vec![];

    validators::validators().traverse(query, schema, &mut errors);

    if errors.is_empty() {
        return Ok(());
    }

    Err(errors)
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::{Emit, IntoDiagnostic};

    pub fn assert_ok(schema: &str, query: &str) {
        let schema_ast = graphql_parser::parse_schema::<&str>(schema).unwrap();
        let query_ast = graphql_parser::parse_query(query).unwrap();

        let result = crate::validate(&schema_ast, &query_ast);

        assert!(
            result.is_ok(),
            "{}",
            result
                .unwrap_err()
                .into_iter()
                .map(|error| format!("{}", error.into_diagnostic().emit(&query).unwrap()))
                .collect::<Vec<_>>()
                .join("\n\n")
        );
    }

    pub fn assert_err(schema: &str, query: &str, expected: &str) {
        let schema = unindent::unindent(schema);
        let query = unindent::unindent(query);
        let expected = unindent::unindent(expected);

        let schema_ast = graphql_parser::parse_schema::<&str>(&schema).unwrap();
        let query_ast = graphql_parser::parse_query(&query).unwrap();

        let result = crate::validate(&schema_ast, &query_ast);
        let errs = result
            .unwrap_err()
            .into_iter()
            .map(|error| format!("{}", error.into_diagnostic().emit(&query).unwrap()))
            .collect::<Vec<_>>()
            .join("\n\n");

        assert_eq!(
            String::from_utf8(strip_ansi_escapes::strip(&errs).unwrap())
                .unwrap()
                .trim(),
            expected.trim(),
            "Actual:\n{}\n\nExpected:\n{}",
            errs,
            expected
        );
    }
}
