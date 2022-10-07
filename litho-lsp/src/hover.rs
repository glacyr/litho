use std::borrow::Borrow;
use std::fmt::Display;

use litho_language::ast::*;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkedString, Position};

use super::{line_col_to_offset, Document};

pub struct HoverProvider<'ast>(&'ast Document);

impl HoverProvider<'_> {
    pub fn new<'ast>(document: &'ast Document) -> HoverProvider<'ast> {
        HoverProvider(document)
    }

    pub fn hover(&self, position: Position) -> Option<Hover> {
        let index = line_col_to_offset(self.0.text(), position.line, position.character);
        let mut hover = None;

        self.0.ast().traverse(&HoverVisitor(index), &mut hover);

        hover
    }
}

struct HoverVisitor(usize);

impl<'ast, T> Visit<'ast, T> for HoverVisitor
where
    T: Borrow<str> + Display,
{
    type Accumulator = Option<Hover>;

    fn visit_object_type_definition(
        &self,
        node: &'ast ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.ty.span().joined(node.name.span()).contains(self.0) {
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
                if field.name.span().contains(self.0) {
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
