use graphql_parser::{parse_query, parse_schema};
use litho_validation::diagnostics::{Emit, IntoDiagnostic};
use litho_validation::validate;
use unindent::unindent;

pub fn main() {
    let schema = parse_schema::<&str>(
        r#"
    scalar Weird

    type Query {
        hello(foo: Boolean!, bar: String!): Hello!
    }

    type Hello {
        world(foo: Boolean!): String!
    }
    "#,
    )
    .unwrap();

    let query_source = unindent(
        &r#"
    query {
        hello(foo: true, bar: "ha") {
            world(foo: true) {
                bar
            }
        }
    }
    "#,
    )
    .replace("\t", "    ");

    let query = parse_query::<&str>(&query_source).unwrap();

    if let Err(errors) = validate(&schema, &query) {
        errors.into_iter().for_each(|error| {
            let message: String = error.into_diagnostic().emit(&query_source).unwrap();
            eprintln!("{}", message)
        })
    }
}
