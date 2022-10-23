use std::sync::Arc;

use litho_language::ast::*;
use litho_types::Database;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{line_col_to_offset, span_to_range, Document, Workspace};

pub struct DefinitionProvider<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
}

impl DefinitionProvider<'_> {
    pub fn new<'a>(document: &'a Document, workspace: &'a Workspace) -> DefinitionProvider<'a> {
        DefinitionProvider {
            document,
            workspace,
        }
    }

    pub fn goto_definition(&self, position: Position) -> Option<GotoDefinitionResponse> {
        let offset = line_col_to_offset(self.document.text(), position.line, position.character);
        let mut definition = None;

        self.document.ast().traverse(
            &DefinitionVisitor {
                document: self.document,
                workspace: self.workspace,
                offset,
            },
            &mut definition,
        );

        definition
    }
}

struct DefinitionVisitor<'a> {
    document: &'a Document,
    workspace: &'a Workspace,
    offset: usize,
}

impl<'a> Visit<'a, SmolStr> for DefinitionVisitor<'a> {
    type Accumulator = Option<GotoDefinitionResponse>;

    fn visit_field(&self, node: &'a Arc<Field<SmolStr>>, accumulator: &mut Self::Accumulator) {
        if let Some(name) = node.name.ok() {
            if name.span().contains(self.offset) {
                if let Some(definition) = self
                    .workspace
                    .database()
                    .inference
                    .field_definitions_by_field
                    .get(&node)
                {
                    if let Some(location) = self.workspace.span_to_location(definition.name.span())
                    {
                        accumulator.replace(GotoDefinitionResponse::Scalar(location));
                    }
                }
            }
        }
    }

    fn visit_arguments(
        &self,
        node: &'a Arc<Arguments<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !node.span().contains(self.offset) {
            return;
        }

        let definitions = match self
            .workspace
            .database()
            .inference
            .definition_for_arguments
            .get(node)
        {
            Some(definitions) => definitions,
            None => return,
        };

        for argument in node.items.iter() {
            if !argument.name.span().contains(self.offset) {
                continue;
            }

            let definition = match definitions
                .definitions
                .iter()
                .find(|definition| definition.name.as_ref() == argument.name.as_ref())
            {
                Some(definition) => definition,
                None => continue,
            };

            if let Some(location) = self.workspace.span_to_location(definition.name.span()) {
                accumulator.replace(GotoDefinitionResponse::Scalar(location));
            }
        }
    }

    fn visit_named_type(&self, node: &'a NamedType<SmolStr>, accumulator: &mut Self::Accumulator) {
        if node.span().contains(self.offset) {
            if let Some(definition) = self
                .workspace
                .database()
                .type_definitions_by_name(node.0.as_ref())
                .next()
            {
                if let Some(location) = self.workspace.span_to_location(definition.name().span()) {
                    accumulator.replace(GotoDefinitionResponse::Scalar(location));
                }
            }
        }
    }
}
