#![no_main]

use std::sync::Arc;

use libfuzzer_sys::fuzz_target;
use litho_language::ast::{Document, Node};
use litho_language::fmt::Format;
use litho_language::Parse;

fuzz_target!(|data: Document<&str>| {
    let string = data.format_to_string(80);
    let data = Arc::new(data);

    assert!(string.len() <= 1_000_000);

    let Ok((parse, rest)) = Document::parse_from_str(Default::default(), &string) else {
        // Recursion protection.
        return;
    };

    let parse = Arc::new(parse);

    assert!(
        parse.congruent(&data),
        "Expected:\n\n{}\n\n... to match:\n\n{}\n\n... and rest {:#?} to be empty",
        parse.format_to_string(80),
        string,
        rest
    );
    assert!(rest.is_empty());
});
