#![feature(test)]

extern crate test;

use std::sync::Arc;

use litho_language::ast::Type;
use litho_language::lex::lexer;
use litho_language::syn::{parser, Stream};
use litho_language::{Document, Parse};
use test::Bencher;

#[bench]
fn test_gitlab_litho(b: &mut Bencher) {
    let string = include_str!("./gitlab.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        assert!(
            Document::<&str>::parse_from_str(Default::default(), &string)
                .unwrap()
                .1
                .is_empty()
        );
    });
}

#[bench]
fn test_gitlab_litho_2(b: &mut Bencher) {
    let string = include_str!("./gitlab.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        let mut lexer = lexer(Default::default(), string);
        parser::<_, &str>(&mut lexer);
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
        assert!(
            Document::<&str>::parse_from_str(Default::default(), &string)
                .unwrap()
                .1
                .is_empty()
        );
    });
}

#[bench]
fn test_kitchen_sink_competitor_async_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./kitchen_sink.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        assert!(async_graphql_parser::parse_query(&string).is_ok());
    });
}

#[bench]
fn test_kitchen_sink_competitor_graphql_parser(b: &mut Bencher) {
    let string = include_str!("./kitchen_sink.graphql");

    assert!(string.len() <= 1_000_000);

    b.iter(|| {
        assert!(graphql_parser::parse_query::<&str>(string).is_ok());
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
