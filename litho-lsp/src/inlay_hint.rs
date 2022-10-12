use std::sync::Arc;

use litho_language::ast::*;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{index_to_position, Document};

pub struct InlayHintProvider<'a>(&'a Document);

impl<'a> InlayHintProvider<'a> {
    pub fn new(document: &'a Document) -> InlayHintProvider<'a> {
        InlayHintProvider(document)
    }

    pub fn inlay_hints(&self) -> impl Iterator<Item = InlayHint> {
        let mut hints = vec![];

        self.0.ast().traverse(&InlayHintVisitor(self.0), &mut hints);

        hints.into_iter()
    }
}

pub struct InlayHintVisitor<'a>(&'a Document);

impl<'a, 'ast> Visit<'ast, SmolStr> for InlayHintVisitor<'a> {
    type Accumulator = Vec<InlayHint>;

    fn visit_selection_set(
        &self,
        node: &'ast Arc<SelectionSet<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = self.0.database().type_by_selection_set(&node) {
            accumulator.push(InlayHint {
                data: None,
                kind: Some(InlayHintKind::TYPE),
                label: InlayHintLabel::String(format!("{}", name)),
                padding_left: Some(false),
                padding_right: Some(true),
                position: index_to_position(self.0.text(), node.braces.0.span().start),
                text_edits: None,
                tooltip: None,
            })
        }
    }
}
