use graphql_parser::parse_schema;
use kono::{AspectExt, Executor, ObjectValue};
use kono_introspection::introspection;

#[tokio::main]
pub async fn main() {
    let schema = parse_schema::<String>(
        r#"
"Comment"
type Query {
	hello: String!
}
"#,
    )
    .unwrap()
    .into_static();

    let (accept, process) = kono::server(
        Executor::new(introspection(schema), ObjectValue::Unit),
        || (),
    );

    futures::future::join(
        warp::serve(kono::server::warp::filter(accept)).run(([127, 0, 0, 1], 3030)),
        process,
    )
    .await;
}
