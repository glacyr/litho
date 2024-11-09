#![feature(test)]

extern crate test;

use std::sync::Arc;

use litho_language::ast::Type;
use litho_language::lex::lexer;
use litho_language::syn::Stream;
use litho_language::Parse;
use test::Bencher;

#[bench]
fn test_ty1(b: &mut Bencher) {
    b.iter(|| {
        let string = "[[[[[[A]!]!]!]]!]";

        assert!(string.len() <= 1_000_000);

        let Ok((parse, rest)) = Arc::<Type<&str>>::parse_from_str(Default::default(), &string)
        else {
            // Recursion protection.
            return;
        };

        assert!(rest.is_empty());
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
