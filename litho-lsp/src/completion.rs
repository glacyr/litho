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

        items.push(Default::default());

        CompletionResponse::Array(items)
    }
}

struct CompletionVisitor<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
    offset: usize,
}

impl<'a> CompletionVisitor<'a> {
    pub fn complete_field_definition(
        &self,
        definition: &'a FieldDefinition<SmolStr>,
    ) -> CompletionItem {
        CompletionItem {
            kind: Some(CompletionItemKind::FIELD),
            label: definition.name.as_ref().to_string(),
            insert_text: Some(format!(
                "{}{}",
                definition.name.as_ref(),
                Printer::snippy_print_arguments_definition(
                    definition.arguments_definition.as_deref()
                )
            )),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            label_details: Some(CompletionItemLabelDetails {
                detail: definition
                    .arguments_definition
                    .as_deref()
                    .map(Printer::print_arguments_definition),
                description: definition.ty.ok().map(|ty| ty.to_string()),
                ..Default::default()
            }),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```\n{}\n```{}",
                    Printer::pretty_print_field(&definition),
                    definition
                        .description
                        .as_ref()
                        .map(|description| format!("\n\n{}", description.0.to_string()))
                        .unwrap_or_default(),
                ),
            })),
            ..Default::default()
        }
    }

    pub fn complete_input_value_definition(
        &self,
        definition: &'a InputValueDefinition<SmolStr>,
    ) -> CompletionItem {
        CompletionItem {
            kind: Some(CompletionItemKind::VARIABLE),
            label: definition.name.to_string(),
            insert_text: Some(format!(
                "{}: ${{0:{}}}",
                definition.name,
                definition
                    .ty
                    .ok()
                    .map(ToString::to_string)
                    .unwrap_or_default()
            )),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            label_details: Some(CompletionItemLabelDetails {
                detail: definition.ty.ok().map(|ty| format!(": {}", ty.to_string())),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl<'a> Visit<'a, SmolStr> for CompletionVisitor<'a> {
    type Accumulator = Vec<CompletionItem>;

    fn visit_selection_set(
        &self,
        node: &'a Arc<SelectionSet<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.span().contains(self.offset) {
            accumulator.truncate(0);

            if let Some(ty) = self.workspace.database().type_by_selection_set(node) {
                accumulator.extend(
                    self.workspace
                        .database()
                        .field_definitions(ty)
                        .map(|def| self.complete_field_definition(def)),
                );
            }
        }
    }

    fn visit_arguments(
        &self,
        node: &'a Arc<Arguments<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.span().contains(self.offset) {
            accumulator.truncate(0);

            if let Some(definition) = self.workspace.database().definition_for_arguments(node) {
                accumulator.extend(
                    definition
                        .definitions
                        .iter()
                        .map(|def| self.complete_input_value_definition(def)),
                )
            }
        }
    }
}
