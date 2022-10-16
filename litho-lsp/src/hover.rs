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

    fn visit_arguments_definition(
        &self,
        node: &'a Arc<ArgumentsDefinition<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        for argument in node.definitions.iter() {
            if !argument.name.span().contains(self.offset) {
                continue;
            }

            accumulator.replace(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "```\n{}: {}\n```\n\n---\n\n{}",
                    argument.name,
                    argument
                        .ty
                        .ok()
                        .map(ToString::to_string)
                        .unwrap_or("...".to_owned()),
                    argument
                        .description
                        .as_ref()
                        .map(|description| description.to_string())
                        .unwrap_or_default(),
                ))),
                range: None,
            });
        }
    }

    fn visit_enum_type_definition(
        &self,
        node: &'a EnumTypeDefinition<SmolStr>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !node.span().contains(self.offset) {
            return;
        }

        if node.name.ok().is_some() && node.name.span().contains(self.offset) {
            accumulator.replace(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "```\nenum {}\n```\n\n---\n\n{}",
                    node.name
                        .ok()
                        .map(ToString::to_string)
                        .unwrap_or("...".to_owned()),
                    node.description
                        .as_ref()
                        .map(|description| description.to_string())
                        .unwrap_or_default(),
                ))),
                range: None,
            });

            return;
        }

        let values = match node.values_definition.as_ref() {
            Some(values) => values,
            None => return,
        };

        for value in values.definitions.iter() {
            if !value.enum_value.span().contains(self.offset) {
                continue;
            }

            accumulator.replace(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "```\nenum {}\n{}\n```\n\n---\n\n{}",
                    node.name
                        .ok()
                        .map(ToString::to_string)
                        .unwrap_or("...".to_owned()),
                    value.enum_value.0,
                    value
                        .description
                        .as_ref()
                        .map(|description| description.to_string())
                        .unwrap_or_default(),
                ))),
                range: None,
            });
        }
    }

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
