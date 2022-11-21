use std::sync::Arc;

use litho_language::ast::*;
use lsp_types::*;
use smol_str::SmolStr;

use super::{Document, Workspace};

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
        let offset = Workspace::position_to_index(self.document.text(), position);
        let mut locations = vec![];

        self.document.ast().traverse(
            &ReferencesVisitor {
                workspace: self.workspace,
                offset,
            },
            &mut locations,
        );

        Some(locations)
    }
}

struct ReferencesVisitor<'a> {
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
