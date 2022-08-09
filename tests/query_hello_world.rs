use kono::{kono, AspectExt};
use serde_json::json;

pub struct HelloWorld;

#[kono]
impl HelloWorld {
    pub fn hello() -> String {
        format!("Welcome!")
    }
}

#[tokio::test]
async fn test_query_hello_world() {
    assert_eq!(
        kono::execute(
            r#"
			query {
				hello
			}
			"#,
            HelloWorld::resolver(),
            &(),
        )
        .await
        .unwrap(),
        json! {{
            "hello": "Welcome!"
        }},
    );
}
