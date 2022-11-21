use litho_language::fmt::Diff;
use lsp_types::TextEdit;

use super::{Document, Workspace};

pub struct FormattingProvider<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
}

impl FormattingProvider<'_> {
    pub fn new<'a>(document: &'a Document, workspace: &'a Workspace) -> FormattingProvider<'a> {
        FormattingProvider {
            document,
            workspace,
        }
    }

    pub fn formatting(&self) -> Vec<TextEdit> {
        Diff::compute(
            self.document.source_id(),
            self.document.text(),
            self.document.ast(),
        )
        .flat_map(|diff| {
            Some(TextEdit {
                range: self.workspace.span_to_range(diff.span)?,
                new_text: diff.replacement,
            })
        })
        .collect()
    }
}
