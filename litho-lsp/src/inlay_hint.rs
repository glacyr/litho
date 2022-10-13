use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{index_to_position, Document};

pub struct InlayHintProvider<'a> {
    document: &'a Document,
    database: &'a Database<SmolStr>,
}

impl<'a> InlayHintProvider<'a> {
    pub fn new(document: &'a Document, database: &'a Database<SmolStr>) -> InlayHintProvider<'a> {
        InlayHintProvider { document, database }
    }

    pub fn inlay_hints(&self) -> impl Iterator<Item = InlayHint> {
        let mut hints = vec![];

        self.document.ast().traverse(
            &InlayHintVisitor {
                document: self.document,
                database: self.database,
            },
            &mut hints,
        );

        hints.into_iter()
    }
}

pub struct InlayHintVisitor<'a> {
    document: &'a Document,
    database: &'a Database<SmolStr>,
}

impl<'a> Visit<'a, SmolStr> for InlayHintVisitor<'a> {
    type Accumulator = Vec<InlayHint>;

    fn visit_selection_set(
        &self,
        node: &'a Arc<SelectionSet<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = self.database.type_by_selection_set(&node) {
            accumulator.push(InlayHint {
                data: None,
                kind: Some(InlayHintKind::TYPE),
                label: InlayHintLabel::String(format!("{}", name)),
                padding_left: Some(false),
                padding_right: Some(true),
                position: index_to_position(self.document.text(), node.braces.0.span().start),
                text_edits: None,
                tooltip: None,
            })
        }
    }
}