use std::sync::Arc;

use litho_language::{Document, Parse};

#[test]
fn test_gitlab_litho() {
    let string = include_str!("../benches/gitlab.graphql");

    assert!(string.len() <= 1_000_000);

    assert!(
        Document::<&str>::parse_from_str(Default::default(), &string)
            .unwrap()
            .1
            .is_empty()
    );
}
