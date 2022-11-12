use crate::lex::{SourceId, Span};

use super::Format;

pub struct Diff {
    pub span: Span,
    pub replacement: String,
}

impl Diff {
    pub fn compute<T>(source_id: SourceId, source: &str, node: &T) -> impl Iterator<Item = Diff>
    where
        T: Format,
    {
        vec![Diff {
            span: Span {
                source_id,
                start: 0,
                end: source.len(),
            },
            replacement: node.format_to_string(80),
        }]
        .into_iter()
    }
}
