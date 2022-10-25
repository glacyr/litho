use std::sync::Arc;

use litho_language::ast::*;
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

    fn keyword(&self, name: &str) -> CompletionItem {
        CompletionItem {
            kind: Some(CompletionItemKind::KEYWORD),
            label: name.to_owned(),
            insert_text: Some(format!("{} ${{1:...}} {{\n\t${{2:...}}\n}}", name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(" ... { ... }".to_owned()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn completion(&self, position: Position) -> CompletionResponse {
        let offset = line_col_to_offset(self.document.text(), position.line, position.character);
        let mut items = vec![
            self.keyword("query"),
            self.keyword("mutation"),
            self.keyword("subscription"),
            self.keyword("type"),
            self.keyword("interface"),
            self.keyword("enum"),
            self.keyword("input"),
        ];

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

    pub fn complete_all_types(&self, input: bool) -> impl Iterator<Item = CompletionItem> + '_ {
        self.workspace
            .database()
            .type_definitions()
            .filter(move |def| match input {
                true => def.is_input(),
                false => def.is_output(),
            })
            .flat_map(|def| {
                def.name().ok().map(|name| CompletionItem {
                    kind: Some(match def {
                        TypeDefinition::EnumTypeDefinition(_)
                        | TypeDefinition::UnionTypeDefinition(_) => CompletionItemKind::ENUM,
                        TypeDefinition::InterfaceTypeDefinition(_) => CompletionItemKind::INTERFACE,
                        TypeDefinition::InputObjectTypeDefinition(_) => CompletionItemKind::STRUCT,
                        TypeDefinition::ScalarTypeDefinition(_) => CompletionItemKind::VALUE,
                        TypeDefinition::ObjectTypeDefinition(_) => CompletionItemKind::CLASS,
                    }),
                    label: name.to_string(),
                    ..Default::default()
                })
            })
    }

    pub fn complete_value(&self, ty: &Type<SmolStr>) -> impl Iterator<Item = CompletionItem> + '_ {
        let mut items = vec![];

        let ty = match ty {
            Type::NonNull(ty) => ty.ty.as_ref(),
            ty => {
                items.push(CompletionItem {
                    label: "null".to_owned(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    ..Default::default()
                });
                ty
            }
        };

        let name = match ty {
            Type::List(_) => {
                // items.push(CompletionItem {
                //     label: "[".to_owned(),
                //     label_details: Some(CompletionItemLabelDetails {
                //         detail: Some("...]".to_owned()),
                //         description: None,
                //     }),
                //     insert_text: Some("[${0}]".to_owned()),
                //     insert_text_format: Some(InsertTextFormat::SNIPPET),
                //     kind: Some(CompletionItemKind::OPERATOR),
                //     ..Default::default()
                // });

                return items.into_iter();
            }
            ty => match ty.name() {
                Some(name) => name,
                None => return items.into_iter(),
            },
        };

        items.extend(
            self.workspace
                .database()
                .enum_value_definitions(name)
                .map(|definition| CompletionItem {
                    label: definition.enum_value.0.as_ref().to_string(),
                    kind: Some(CompletionItemKind::ENUM),
                    ..Default::default()
                }),
        );

        match self
            .workspace
            .database()
            .type_definitions_by_name(name)
            .next()
        {
            Some(ty) if ty.is_scalar() && name == "String" => {
                items.push(CompletionItem {
                    label: "\"".to_owned(),
                    label_details: Some(CompletionItemLabelDetails {
                        detail: Some("...\"".to_owned()),
                        description: None,
                    }),
                    insert_text: Some("\"${0}\"".to_owned()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    kind: Some(CompletionItemKind::OPERATOR),
                    ..Default::default()
                });
            }
            Some(ty) if ty.is_input_object_type() => {
                items.push(CompletionItem {
                    label: "{".to_owned(),
                    label_details: Some(CompletionItemLabelDetails {
                        detail: Some("...}".to_owned()),
                        description: None,
                    }),
                    insert_text: Some("{${0}}".to_owned()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    kind: Some(CompletionItemKind::OPERATOR),
                    ..Default::default()
                });
            }
            _ => {}
        };

        items.into_iter()
    }
}

impl<'a> Visit<'a, SmolStr> for CompletionVisitor<'a> {
    type Accumulator = Vec<CompletionItem>;

    fn visit_definition(&self, node: &'a Definition<SmolStr>, accumulator: &mut Self::Accumulator) {
        if node.span().contains(self.offset) {
            accumulator.truncate(0);
        }
    }

    fn visit_selection_set(
        &self,
        node: &'a Arc<SelectionSet<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if node.span().contains(self.offset) {
            accumulator.truncate(0);

            if let Some(ty) = self
                .workspace
                .database()
                .inference
                .type_by_selection_set
                .get(node)
            {
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

            if let Some(definition) = self
                .workspace
                .database()
                .inference
                .definition_for_arguments
                .get(node)
            {
                accumulator.extend(
                    definition
                        .definitions
                        .iter()
                        .map(|def| self.complete_input_value_definition(def)),
                )
            }
        }
    }

    fn visit_fields_definition(
        &self,
        node: &'a FieldsDefinition<SmolStr>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !node.span().contains(self.offset) {
            return;
        }

        for field in node.definitions.iter() {
            if field.span().collapse_to_start().before(self.offset) {
                accumulator.truncate(0);

                if field.colon.ok().is_some() && field.colon.span().before(self.offset) {
                    accumulator.extend(self.complete_all_types(false))
                }
            }
        }
    }

    fn visit_input_fields_definition(
        &self,
        node: &'a InputFieldsDefinition<SmolStr>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !node.span().contains(self.offset) {
            return;
        }

        for field in node.definitions.iter() {
            if field.span().collapse_to_start().before(self.offset) {
                accumulator.truncate(0);

                if field.colon.ok().is_some() && field.colon.span().before(self.offset) {
                    accumulator.extend(self.complete_all_types(true))
                }

                match field.default_value.as_ref() {
                    Some(default_value) if default_value.eq.span().before(self.offset) => {
                        accumulator.truncate(0);

                        if let Some(ty) = field.ty.ok() {
                            accumulator.extend(self.complete_value(ty))
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn visit_arguments_definition(
        &self,
        node: &'a Arc<ArgumentsDefinition<SmolStr>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if !node.span().contains(self.offset) {
            return;
        }

        for field in node.definitions.iter() {
            if field.span().collapse_to_start().before(self.offset) {
                accumulator.truncate(0);

                if field.colon.ok().is_some() && field.colon.span().before(self.offset) {
                    accumulator.extend(self.complete_all_types(true))
                }
            }
        }
    }

    fn visit_value(&self, node: &'a Arc<Value<SmolStr>>, accumulator: &mut Self::Accumulator) {
        match node.as_ref() {
            Value::ListValue(list) if list.brackets.span().contains(self.offset) => {
                accumulator.truncate(0);

                match self
                    .workspace
                    .database()
                    .inference
                    .types_for_values
                    .get(node)
                    .map(AsRef::as_ref)
                {
                    Some(Type::List(ty)) => match ty.ty.ok() {
                        Some(ty) => accumulator.extend(self.complete_value(ty)),
                        None => {}
                    },
                    Some(_) | None => {}
                }
            }
            _ => {}
        }
    }
}
