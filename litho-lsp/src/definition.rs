use litho_language::ast::*;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{line_col_to_offset, span_to_range, Document};

pub struct DefinitionProvider<'ast>(&'ast Document);

impl DefinitionProvider<'_> {
    pub fn new<'ast>(document: &'ast Document) -> DefinitionProvider<'ast> {
        DefinitionProvider(document)
    }

    pub fn goto_definition(&self, position: Position) -> Option<GotoDefinitionResponse> {
        let index = line_col_to_offset(self.0.text(), position.line, position.character);
        let mut definition = None;

        self.0
            .ast()
            .traverse(&DefinitionVisitor(self.0, index), &mut definition);

        definition
    }
}

struct DefinitionVisitor<'ast>(&'ast Document, usize);

impl<'ast, 'a> Visit<'ast, SmolStr> for DefinitionVisitor<'ast> {
    type Accumulator = Option<GotoDefinitionResponse>;

    fn visit_named_type(
        &self,
        node: &'ast NamedType<SmolStr>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.span().contains(self.1) {
            eprintln!("Searching definition of: {:?}", node.0.as_ref());
            eprintln!("Database: {:#?}", self.0.database());
            if let Some(definition) = self
                .0
                .database()
                .type_definitions_by_name(node.0.as_ref())
                .next()
            {
                eprintln!("Found definition: {:#?}", definition);

                accumulator.replace(GotoDefinitionResponse::Scalar(Location {
                    uri: self.0.url().clone(),
                    range: span_to_range(self.0.text(), definition.name().span()),
                }));
            }
        }
    }
}
