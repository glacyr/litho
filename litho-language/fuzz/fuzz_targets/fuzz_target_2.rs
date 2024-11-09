#![no_main]

use std::sync::Arc;

use libfuzzer_sys::fuzz_target;
use litho_language::ast::{Node, Type};
use litho_language::fmt::Format;
use litho_language::lex::lexer;
use litho_language::syn::executable::ty2;
use litho_language::syn::Stream;
use litho_language::Parse;

fuzz_target!(|data: Type<&str>| {
    let string = data.format_to_string(80);
    let data = Arc::new(data);

    assert!(string.len() <= 1_000_000);

    let lexer = lexer(Default::default(), &string).exact();
    let stream = Stream::from(&lexer);

    let (rest, parse) = ty2(stream);
    let rest = rest.into_unexpected::<Vec<_>>();

    let parse = Arc::new(parse);

    assert!(
        parse.congruent(&data),
        "Expected {:#?} to match {:#?} and rest {:#?} to be empty",
        parse,
        data,
        rest
    );
    assert!(rest.is_empty());
});
