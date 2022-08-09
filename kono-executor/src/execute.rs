use graphql_parser::parse_query;
use serde_json::{to_value, Value};

use super::{Executor, Resolver};

/// Executes the operation in the given document with the given resolver and
/// context.
pub async fn execute<R>(
    document: &str,
    resolver: R,
    context: &R::Context,
) -> Result<Value, R::Error>
where
    R: Resolver,
{
    let executor = Executor::new(resolver);
    let document = parse_query(document).unwrap();
    let result = executor
        .execute_request(document, None, &Default::default(), context)
        .await?;
    Ok(to_value(result).unwrap())
}
