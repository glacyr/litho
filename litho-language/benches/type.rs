#![feature(test)]

extern crate test;

use bumpalo::Bump;
use litho_language::ast::{BumpaloContext, ExecutableDocument, TypeSystemDocument};
use litho_language::Parse;
use test::Bencher;

#[bench]
fn test_github_litho(b: &mut Bencher) {
    let string = include_str!("./github.graphql");

    b.iter(|| {
        let bump = Bump::with_capacity(1024 * 128);

        assert!(TypeSystemDocument::<&str>::parse_from_str(
            Default::default(),
            &string,
            BumpaloContext::new(&bump)
        )
        .unwrap()
        .1
        .is_empty());
    });
}

#[bench]
fn test_github_competitor_async_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./github.graphql");

    b.iter(|| {
        assert!(async_graphql_parser::parse_schema(&string).is_ok());
    });
}

#[bench]
fn test_github_competitor_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./github.graphql");

    b.iter(|| {
        assert!(graphql_parser::parse_schema::<&str>(string).is_ok());
    });
}

#[bench]
fn test_gitlab_litho(b: &mut Bencher) {
    let string = include_str!("./gitlab.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        let bump = Bump::with_capacity(1024 * 4);

        assert!(TypeSystemDocument::<&str>::parse_from_str(
            Default::default(),
            &string,
            BumpaloContext::new(&bump)
        )
        .unwrap()
        .1
        .is_empty());
    });
}

#[bench]
fn test_gitlab_competitor_async_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./gitlab.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        assert!(async_graphql_parser::parse_schema(&string).is_ok());
    });
}

#[bench]
fn test_gitlab_competitor_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./gitlab.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        assert!(graphql_parser::parse_schema::<&str>(string).is_ok());
    });
}

#[bench]
fn test_kitchen_sink_litho(b: &mut Bencher) {
    let string = include_str!("./kitchen_sink.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        let bump = Bump::with_capacity(1024 * 4);

        for _ in 0..1000 {
            assert!(ExecutableDocument::<&str>::parse_from_str(
                Default::default(),
                &string,
                BumpaloContext::new(&bump)
            )
            .unwrap()
            .1
            .is_empty());
        }
    });
}

#[bench]
fn test_kitchen_sink_competitor_async_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./kitchen_sink.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        for _ in 0..1000 {
            assert!(async_graphql_parser::parse_query(&string).is_ok());
        }
    });
}

#[bench]
fn test_kitchen_sink_competitor_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./kitchen_sink.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        for _ in 0..1000 {
            assert!(graphql_parser::parse_query::<&str>(string).is_ok());
        }
    });
}

#[bench]
fn test_kitchen_sink_competitor_stellate(b: &mut Bencher) {
    use graphql_query::ast::{ASTContext, Document, ParseNode, PrintNode};

    let string = include_str!("./kitchen_sink.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        for _ in 0..1000 {
            let ctx = ASTContext::new();
            // Parse the source_string with the context
            assert!(Document::parse(&ctx, string).is_ok());
        }
    });
}

// #[bench]
// fn test_ty2(b: &mut Bencher) {
//     b.iter(|| {
//         let string = "[[[[[[A]!]!]!]]!]";

//         assert!(string.len() <= 1_000_000);

//         let lexer = lexer::<&str>(Default::default(), &string).exact();
//         let stream = Stream::from(&lexer);

//         let (rest, parse) = ty2(stream).unwrap();
//         let rest = rest.into_unexpected::<Vec<_>>();

//         assert!(rest.is_empty());
//     });
// }
