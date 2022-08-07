use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::net::SocketAddr;

use futures::channel::{mpsc, oneshot};
use futures::StreamExt;
use graphql_parser::parse_query;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use kono_aspect::{Error, ObjectValue, RecordResolver};
use kono_executor::{Executor, Resolver};
use kono_introspection::introspection;
use kono_schema::{Emit, Schema};

#[derive(Debug, Deserialize)]
pub struct Request {
    pub operation_name: Option<String>,
    pub query: String,

    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct Response<E> {
    pub data: Option<IndexMap<String, serde_json::Value>>,
    pub errors: Vec<E>,
}

pub type Accept<E> = mpsc::Sender<(Request, oneshot::Sender<Response<E>>)>;

async fn process<R, F>(
    executor: &Executor<R>,
    request: Request,
    context_fn: &F,
) -> Response<R::Error>
where
    R: Resolver,
    F: Fn() -> R::Context,
{
    let document = parse_query(&request.query).unwrap();

    match executor
        .execute_request(
            document,
            request.operation_name.as_ref().map(|name| name.as_str()),
            &request.variables,
            &context_fn(),
        )
        .await
    {
        Ok(data) => Response {
            data: Some(data),
            errors: vec![],
        },
        Err(error) => Response {
            data: None,
            errors: vec![error],
        },
    }
}

pub fn server<R, F>(resolver: R, context_fn: F) -> (Accept<R::Error>, impl Future)
where
    R: Resolver<Error = Error, Value = ObjectValue> + Schema,
    R::Context: 'static,
    F: Fn() -> R::Context,
{
    let schema = resolver.schema();
    let executor = Executor::new((
        RecordResolver::default(),
        resolver,
        introspection(schema.emit()),
    ));
    let (sender, mut receiver) = mpsc::channel(1024);

    (sender, async move {
        while let Some((request, channel)) = receiver.next().await {
            channel
                .send(process(&executor, request, &context_fn).await)
                .unwrap();
        }
    })
}

pub async fn serve<R, F>(resolver: R, context_fn: F, address: impl Into<SocketAddr>)
where
    R: Resolver<Error = Error, Value = ObjectValue> + Schema,
    R::Context: 'static,
    F: Fn() -> R::Context,
{
    let (accept, process) = server(resolver, context_fn);

    futures::future::join(::warp::serve(warp::filter(accept)).run(address), process).await;
}

#[cfg(feature = "warp")]
pub mod warp {
    use super::{Accept, Request};

    use futures::channel::oneshot;
    use futures::SinkExt;
    use http::Response;
    use serde::Serialize;
    use serde_json::to_string_pretty;
    use warp::{Filter, Rejection};

    pub fn filter<E>(
        accept: Accept<E>,
    ) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone
    where
        E: Serialize + Send,
    {
        warp::filters::body::json().and_then(move |request: Request| {
            let mut accept = accept.clone();

            async move {
                let (sender, receiver) = oneshot::channel();

                accept.send((request, sender)).await.unwrap();

                let result = receiver.await.unwrap();

                Ok::<Response<String>, Rejection>(
                    Response::builder()
                        .header("Content-Type", "application/json")
                        .header("Server", "kono/1.0.0")
                        .body(to_string_pretty(&result).unwrap())
                        .unwrap(),
                )
            }
        })
    }
}
