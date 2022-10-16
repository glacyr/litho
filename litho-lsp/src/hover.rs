use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;
use smol_str::SmolStr;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkedString, Position};

use super::{line_col_to_offset, Document, Printer};

pub struct HoverProvider<'a> {
    document: &'a Document,
    database: &'a Database<SmolStr>,
}

impl HoverProvider<'_> {
    pub fn new<'a>(document: &'a Document, database: &'a Database<SmolStr>) -> HoverProvider<'a> {
        HoverProvider { document, database }
    }

    pub fn hover(&self, position: Position) -> Option<Hover> {
        let offset = line_col_to_offset(self.document.text(), position.line, position.character);
        let mut hover = None;

        self.document.ast().traverse(
            &HoverVisitor {
                document: self.document,
                database: self.database,
                offset,
            },
            &mut hover,
        );

        hover
    }
}

struct HoverVisitor<'a> {
    document: &'a Document,
    database: &'a Database<SmolStr>,
    offset: usize,
}

impl<'a> Visit<'a, SmolStr> for HoverVisitor<'a> {
    type Accumulator = Option<Hover>;

    fn visit_field(&self, node: &'a Arc<Field<SmolStr>>, accumulator: &mut Self::Accumulator) {
        if let Some(name) = node.name.ok() {
            if name.span().contains(self.offset) {
                if let Some(definition) = self.database.field_definitions_by_field(&node).next() {
                    accumulator.replace(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(format!(
                            "```\n{}\n```\n\n---\n\n{}",
                            Printer::pretty_print_field(definition),
                            definition
                                .description
                                .as_ref()
                                .map(|description| description.to_string())
                                .unwrap_or_default(),
                        ))),
                        range: None,
                    });
                }
            }
        }
    }

    fn visit_named_type(&self, node: &'a NamedType<SmolStr>, accumulator: &mut Self::Accumulator) {
        if !node.span().contains(self.offset) {
            return;
        }

        if let Some(definition) = self
            .database
            .type_definitions_by_name(node.0.as_ref())
            .next()
        {
            accumulator.replace(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "```\n{}\n```\n\n---\n\n{}",
                    Printer::short_print_type_definition(definition),
                    definition
                        .description()
                        .as_ref()
                        .map(|description| description.to_string())
                        .unwrap_or_default()
                ))),
                range: None,
            });
        }
    }

    fn visit_object_type_definition(
        &self,
        node: &'a ObjectTypeDefinition<SmolStr>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node
            .ty
            .span()
            .joined(node.name.span())
            .contains(self.offset)
        {
            accumulator.replace(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "```\ntype {}\n```\n\n---\n\n{}",
                    node.name,
                    node.description
                        .as_ref()
                        .map(|description| description.to_string())
                        .unwrap_or_default()
                ))),
                range: None,
            });
        } else {
            for field in node
                .fields_definition
                .iter()
                .flat_map(|fields| fields.definitions.iter())
            {
                if field.name.span().contains(self.offset) {
                    accumulator.replace(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(format!(
                            "```\ntype {}\n{}{}: {}\n```\n\n---\n\n{}",
                            node.name,
                            field.name,
                            Printer::pretty_print_arguments_definition(
                                field.arguments_definition.as_deref()
                            ),
                            field
                                .ty
                                .ok()
                                .map(ToString::to_string)
                                .unwrap_or("...".to_owned()),
                            field
                                .description
                                .as_ref()
                                .map(|description| description.to_string())
                                .unwrap_or_default(),
                        ))),
                        range: None,
                    });
                }
            }
        }
    }
}
