use std::sync::Arc;

use litho_language::ast::*;
use lsp_types::*;
use smol_str::SmolStr;

use super::{Document, Workspace};

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
        let offset = Workspace::position_to_index(self.document.text(), position);
        let mut definition = None;

        self.document.ast().traverse(
            &DefinitionVisitor {
                workspace: self.workspace,
                offset,
            },
            &mut definition,
        );

        definition
    }
}

struct DefinitionVisitor<'a> {
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

        let Some(definitions) = self
            .workspace
            .database()
            .inference
            .definition_for_arguments
            .get(node) else {
            return
        };

        for argument in node.items.iter() {
            if !argument.name.span().contains(self.offset) {
                continue;
            }

            let Some(definition) = definitions
                .definitions
                .iter()
                .find(|definition| definition.name.as_ref() == argument.name.as_ref()) else {
                continue
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

    fn visit_value(&self, node: &'a Arc<Value<SmolStr>>, accumulator: &mut Self::Accumulator) {
        if !node.span().contains(self.offset) {
            return;
        }

        match node.as_ref() {
            Value::EnumValue(value) => {
                let def = self
                    .workspace
                    .database()
                    .inference
                    .types_for_values
                    .get(node)
                    .and_then(|ty| ty.name())
                    .and_then(|ty| {
                        self.workspace
                            .database()
                            .enum_value_definitions_by_name(ty, value.0.as_ref())
                            .next()
                    });

                accumulator.replace(GotoDefinitionResponse::Array(
                    def.into_iter()
                        .flat_map(|def| self.workspace.span_to_location(def.enum_value.span()))
                        .collect(),
                ));
            }
            Value::Variable(_) => {
                accumulator.replace(GotoDefinitionResponse::Array(
                    self.workspace
                        .database()
                        .inference
                        .definitions_for_variable
                        .get(node)
                        .flat_map(|definition| {
                            self.workspace.span_to_location(definition.variable.span())
                        })
                        .collect(),
                ));
            }
            _ => {}
        }
    }
}
