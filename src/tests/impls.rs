use crate as kono;

use kono::{kono, AspectExt};

#[test]
fn test_impl_fields() {
    pub struct Example;

    #[kono]
    impl Example {
        fn first_name(&self) -> &str {
            "Tim"
        }
    }

    kono::tests::test_eq(
        Example::resolver(),
        r#"
    type Example {
      __typename: String!
      firstName: String!
    }

    type Query {
      __typename: String!
    }
	"#,
    )
}

#[test]
fn test_impl_queries() {
    pub struct Example;

    #[kono]
    impl Example {
        fn example() -> Example {
            Example
        }
    }

    kono::tests::test_eq(
        Example::resolver(),
        r#"
    type Example {
      __typename: String!
    }

    type Query {
      __typename: String!
      example: Example!
    }
	"#,
    )
}

#[test]
fn test_impl_queries_with_inputs() {
    pub struct Example;

    #[kono]
    impl Example {
        fn example(id: String) -> Example {
            let _ = id;

            Example
        }
    }

    kono::tests::test_eq(
        Example::resolver(),
        r#"
    type Example {
      __typename: String!
    }

    type Query {
      __typename: String!
      example(id: String!): Example!
    }
	"#,
    )
}
