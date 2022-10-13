use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{line_col_to_offset, Document, Printer, Workspace};

pub struct CompletionProvider<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
}

impl CompletionProvider<'_> {
    pub fn new<'a>(document: &'a Document, workspace: &'a Workspace) -> CompletionProvider<'a> {
        CompletionProvider {
            document,
            workspace,
        }
    }

    pub fn completion(&self, position: Position) -> CompletionResponse {
        let offset = line_col_to_offset(self.document.text(), position.line, position.character);
        let mut items = vec![];

        self.document.ast().traverse(
            &CompletionVisitor {
                document: self.document,
                workspace: self.workspace,
                offset,
            },
            &mut items,
        );

        CompletionResponse::Array(items)
    }
}

struct CompletionVisitor<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
    offset: usize,
}

impl<'a> Visit<'a, SmolStr> for CompletionVisitor<'a> {
    type Accumulator = Vec<CompletionItem>;

    fn visit_selection_set(
        &self,
        node: &'a Arc<SelectionSet<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.span().contains(self.offset) {
            if let Some(ty) = self.workspace.database().type_by_selection_set(node) {
                accumulator.truncate(0);
                accumulator.extend(self.workspace.database().field_definitions(ty).map(|def| {
                    CompletionItem {
                        kind: Some(CompletionItemKind::FIELD),
                        label: def.name.as_ref().to_string(),
                        insert_text: Some(format!(
                            "{}{}",
                            def.name.as_ref(),
                            Printer::snippy_print_arguments_definition(&def.arguments_definition)
                        )),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        label_details: Some(CompletionItemLabelDetails {
                            detail: def.arguments_definition.as_ref().map(|def| {
                                format!(
                                    "({})",
                                    def.definitions
                                        .iter()
                                        .map(|arg| format!(
                                            "{}: {}",
                                            arg.name,
                                            arg.ty
                                                .ok()
                                                .map(|ty| ty.to_string())
                                                .unwrap_or("...".to_owned())
                                        ))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            }),
                            description: def.ty.ok().map(|ty| ty.to_string()),
                            ..Default::default()
                        }),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!(
                                "```\n{}\n```{}",
                                Printer::pretty_print_field(&def),
                                def.description
                                    .as_ref()
                                    .map(|description| format!("\n\n{}", description.0.to_string()))
                                    .unwrap_or_default(),
                            ),
                        })),
                        ..Default::default()
                    }
                }));
            }
        }
    }
}
