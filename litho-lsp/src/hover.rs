use std::sync::Arc;

use litho_language::ast::*;
use smol_str::SmolStr;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkedString, Position};

use super::{line_col_to_offset, Document, Printer};

pub struct HoverProvider<'ast>(&'ast Document);

impl HoverProvider<'_> {
    pub fn new<'ast>(document: &'ast Document) -> HoverProvider<'ast> {
        HoverProvider(document)
    }

    pub fn hover(&self, position: Position) -> Option<Hover> {
        let index = line_col_to_offset(self.0.text(), position.line, position.character);
        let mut hover = None;

        self.0
            .ast()
            .traverse(&HoverVisitor(self.0, index), &mut hover);

        hover
    }
}

struct HoverVisitor<'a>(&'a Document, usize);

impl<'a, 'ast> Visit<'ast, SmolStr> for HoverVisitor<'a> {
    type Accumulator = Option<Hover>;

    fn visit_field(&self, node: &'ast Arc<Field<SmolStr>>, accumulator: &mut Self::Accumulator) {
        if let Some(name) = node.name.ok() {
            if name.span().contains(self.1) {
                if let Some(definition) = self.0.database().field_definitions_by_field(&node).next()
                {
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

    fn visit_named_type(
        &self,
        node: &'ast NamedType<SmolStr>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !node.span().contains(self.1) {
            return;
        }

        if let Some(definition) = self
            .0
            .database()
            .type_definitions_by_name(node.0.as_ref())
            .next()
        {
            accumulator.replace(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "```\ntype {}\n```\n\n---\n\n{}",
                    definition.name(),
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
        node: &'ast ObjectTypeDefinition<SmolStr>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.ty.span().joined(node.name.span()).contains(self.1) {
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
                if field.name.span().contains(self.1) {
                    accumulator.replace(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(format!(
                            "```\ntype {}\n{}: ...\n```\n\n---\n\n{}",
                            node.name,
                            field.name,
                            field
                                .description
                                .as_ref()
                                .map(|description| description.to_string())
                                .unwrap_or_default()
                        ))),
                        range: None,
                    });
                }
            }
        }
    }
}
