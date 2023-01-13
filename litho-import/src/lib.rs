use std::collections::HashMap;
use std::future::Future;

use litho_language::fmt::Format;
use litho_language::Document;

mod introspection;

use introspection::Response;
use reqwest::header::HeaderMap;

pub trait Importer {
    type Error: ToString;
    type Future<'a, T>: Future<Output = Result<T, Self::Error>> + 'a
    where
        Self: 'a;

    fn import<'a, T>(&'a self, path: &'a str) -> Self::Future<'a, T>;
}

pub async fn import<T>(url: &str, headers: HeaderMap) -> Result<T, String>
where
    T: for<'a> From<&'a str>,
{
    let mut params = HashMap::new();
    params.insert("query", include_str!("../introspection.graphql"));
    params.insert("operationName", "IntrospectionQuery");

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .headers(headers)
        .json(&params)
        .send()
        .await
        .map_err(|err| err.to_string())?;
    let json = response
        .json::<Response>()
        .await
        .map_err(|err| err.to_string())?;

    let node: Document<String> = json.data.schema.into();

    Ok(T::from(&node.format_to_string(80)))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_import() {
        let result = super::import::<String>("https://api.spacex.land/graphql", Default::default())
            .await
            .unwrap();
        eprintln!("{}", result);
    }
}
