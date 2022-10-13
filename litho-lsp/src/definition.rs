use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{line_col_to_offset, span_to_range, Document};

pub struct DefinitionProvider<'a> {
    document: &'a Document,
    database: &'a Database<SmolStr>,
}

impl DefinitionProvider<'_> {
    pub fn new<'a>(
        document: &'a Document,
        database: &'a Database<SmolStr>,
    ) -> DefinitionProvider<'a> {
        DefinitionProvider { document, database }
    }

    pub fn goto_definition(&self, position: Position) -> Option<GotoDefinitionResponse> {
        let offset = line_col_to_offset(self.document.text(), position.line, position.character);
        let mut definition = None;

        self.document.ast().traverse(
            &DefinitionVisitor {
                document: self.document,
                database: self.database,
                offset,
            },
            &mut definition,
        );

        definition
    }
}

struct DefinitionVisitor<'a> {
    document: &'a Document,
    database: &'a Database<SmolStr>,
    offset: usize,
}

impl<'a> Visit<'a, SmolStr> for DefinitionVisitor<'a> {
    type Accumulator = Option<GotoDefinitionResponse>;

    fn visit_field(&self, node: &'a Arc<Field<SmolStr>>, accumulator: &mut Self::Accumulator) {
        if let Some(name) = node.name.ok() {
            if name.span().contains(self.offset) {
                if let Some(definition) = self.database.field_definitions_by_field(&node).next() {
                    accumulator.replace(GotoDefinitionResponse::Scalar(Location {
                        uri: self.document.url().clone(),
                        range: span_to_range(self.document.text(), definition.name.span()),
                    }));
                }
            }
        }
    }

    fn visit_named_type(&self, node: &'a NamedType<SmolStr>, accumulator: &mut Self::Accumulator) {
        if node.span().contains(self.offset) {
            if let Some(definition) = self
                .database
                .type_definitions_by_name(node.0.as_ref())
                .next()
            {
                accumulator.replace(GotoDefinitionResponse::Scalar(Location {
                    uri: self.document.url().clone(),
                    range: span_to_range(self.document.text(), definition.name().span()),
                }));
            }
        }
    }
}
