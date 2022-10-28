use std::sync::Arc;

use litho_language::ast::*;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{line_col_to_offset, Document, Workspace};

pub struct ReferencesProvider<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
}

impl ReferencesProvider<'_> {
    pub fn new<'a>(document: &'a Document, workspace: &'a Workspace) -> ReferencesProvider<'a> {
        ReferencesProvider {
            document,
            workspace,
        }
    }

    pub fn references(&self, position: Position) -> Option<Vec<Location>> {
        let offset = line_col_to_offset(self.document.text(), position.line, position.character);
        let mut locations = vec![];

        self.document.ast().traverse(
            &ReferencesVisitor {
                document: self.document,
                workspace: self.workspace,
                offset,
            },
            &mut locations,
        );

        Some(locations)
    }
}

struct ReferencesVisitor<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
    offset: usize,
}

impl<'a> Visit<'a, SmolStr> for ReferencesVisitor<'a> {
    type Accumulator = Vec<Location>;

    fn visit_fragment_definition(
        &self,
        node: &'a Arc<FragmentDefinition<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !node.fragment_name.span().contains(self.offset) {
            return;
        }

        accumulator.extend(
            self.workspace
                .database()
                .usages
                .fragments
                .usages(node)
                .flat_map(|usage| self.workspace.span_to_location(usage.fragment_name.span())),
        )
    }
}
