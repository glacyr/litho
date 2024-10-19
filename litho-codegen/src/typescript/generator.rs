use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use litho_language::ast::{Type, TypeDefinition};
use litho_language::lex::{Name, SourceId};
use litho_types::Database;
use serde::Deserialize;

use super::{LineIndex, SourceMapped};

#[derive(Deserialize)]
pub struct LithoExportTypescript {
    #[serde(default)]
    pub skip: bool,
    pub prepend: Option<String>,
    pub rewrite: Option<String>,
}

pub struct Generator<'a, T>
where
    T: Eq + Hash,
{
    database: &'a Database<T>,
    pub js: SourceMapped<'a>,
    pub dts: SourceMapped<'a>,
}

impl<'a, T> Generator<'a, T>
where
    T: Eq + Hash + Borrow<str>,
{
    pub fn new(
        database: &'a Database<T>,
        source_map: &'a HashMap<SourceId, (&'a str, LineIndex)>,
    ) -> Generator<'a, T> {
        let js = SourceMapped::new(&source_map);
        let dts = SourceMapped::new(&source_map);

        Generator { database, js, dts }.generate()
    }

    fn preprocess_export_directives(&mut self) {
        let directives = self
            .database
            .schema_directives()
            .filter(|directive| {
                directive
                    .name
                    .ok()
                    .map(|name| name.as_ref().borrow() == "litho_export")
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        for directive in directives {
            let Some(typescript) = directive.argument("typescript") else {
                continue;
            };

            let Some(value) = typescript.value.ok().and_then(|value| value.to_json()) else {
                continue;
            };

            let Ok(value) = serde_json::from_value::<LithoExportTypescript>(value) else {
                continue;
            };

            if let Some(prepend) = value.prepend.as_ref() {
                self.dts.text(prepend);
            }
        }
    }

    fn write_type(&mut self, ty: &Type<T>) {
        match ty {
            Type::List(ty) => {
                match ty.ty.ok() {
                    Some(ty) => self.write_type(ty),
                    None => {
                        self.dts.text("never");
                    }
                }

                self.dts.text("[]");
            }
            Type::Named(named) => {
                match named.0.as_raw_token().source.borrow() {
                    "Boolean" => self.dts.text("boolean"),
                    "Int" | "Float" => self.dts.text("number"),
                    "String" | "ID" => self.dts.text("string"),
                    _ => self.dts.token(named.0.as_raw_token()),
                };
            }
            Type::NonNull(ty) => {
                self.write_type(&ty.ty);
            }
        }
    }

    fn process_type_definition(&mut self, ty: &TypeDefinition<T>) {
        let Some(name) = ty.name().ok() else { return };

        let directives = self
            .database
            .type_directives(name.as_ref())
            .filter(|directive| {
                directive
                    .name
                    .ok()
                    .map(|name| name.as_ref().borrow() == "litho_export")
                    .unwrap_or_default()
            });

        for directive in directives {
            let Some(typescript) = directive.argument("typescript") else {
                continue;
            };

            let Some(value) = typescript.value.ok().and_then(|value| value.to_json()) else {
                continue;
            };

            let Ok(value) = serde_json::from_value::<LithoExportTypescript>(value) else {
                continue;
            };

            if value.skip {
                return;
            }
        }

        match ty {
            TypeDefinition::EnumTypeDefinition(_) => self.process_enum(name),
            TypeDefinition::InputObjectTypeDefinition(_) => self.process_input_object(name),
            TypeDefinition::InterfaceTypeDefinition(_) => self.process_interface(name),
            TypeDefinition::ObjectTypeDefinition(_) => self.process_object(name),
            TypeDefinition::ScalarTypeDefinition(_) => self.process_scalar(name),
            TypeDefinition::UnionTypeDefinition(_) => self.process_union(name),
        }
    }

    fn process_enum(&mut self, name: &Name<T>) {
        self.js
            .text("exports.")
            .token(name.as_raw_token())
            .text(" = {\n");

        self.dts
            .text("export enum ")
            .token(name.as_raw_token())
            .text(" {\n");

        let values = self.database.enum_value_definitions(name.as_ref());

        for value in values {
            self.js
                .text("    ")
                .token(value.enum_value.0.as_raw_token())
                .text(": \"")
                .token(value.enum_value.0.as_raw_token())
                .text("\",\n");

            self.dts
                .text("    ")
                .token(value.enum_value.0.as_raw_token())
                .text(" = \"")
                .token(value.enum_value.0.as_raw_token())
                .text("\",\n");
        }

        self.js.text("}\n\n");

        self.dts.text("}\n\n");
    }

    fn process_input_object(&mut self, name: &Name<T>) {
        self.dts
            .text("export interface ")
            .token(name.as_raw_token())
            .text(" {\n");

        for field in self.database.input_value_definitions(name.as_ref()) {
            self.dts.text("    ").token(field.name.as_raw_token());

            match field.ty.ok() {
                Some(ty) if ty.is_nullable() => {
                    self.dts.text("?");
                }
                Some(_) | None => {}
            }

            self.dts.text(": ");

            match field.ty.ok() {
                Some(ty) => self.write_type(ty),
                None => {
                    self.dts.text("never");
                }
            }

            self.dts.text(",\n");
        }

        self.dts.text("}\n\n");
    }

    fn process_interface(&mut self, name: &Name<T>) {
        self.dts
            .text("export interface ")
            .token(name.as_raw_token())
            .text(" {\n");

        for field in self.database.field_definitions(name.as_ref()) {
            self.dts.text("    ").token(field.name.as_raw_token());

            match field.ty.ok() {
                Some(ty) if ty.is_nullable() => {
                    self.dts.text("?");
                }
                Some(_) | None => {}
            }

            self.dts.text(": ");

            match field.ty.ok() {
                Some(ty) => self.write_type(ty),
                None => {
                    self.dts.text("never");
                }
            }

            match field.ty.ok() {
                Some(ty) if ty.is_nullable() => {
                    self.dts.text(" | null");
                }
                Some(_) | None => {}
            }

            self.dts.text(",\n");
        }

        self.dts.text("}\n\n");
    }

    fn process_object(&mut self, name: &Name<T>) {
        self.js
            .text("exports.")
            .token(name.as_raw_token())
            .text(" = class ")
            .token(name.as_raw_token())
            .text(" {}\n\n");

        self.dts
            .text("export class ")
            .token(name.as_raw_token())
            .text(" {\n");

        for field in self.database.field_definitions(name.as_ref()) {
            self.dts.text("    ").token(field.name.as_raw_token());

            match field.ty.ok() {
                Some(ty) if ty.is_nullable() => {
                    self.dts.text("?");
                }
                Some(_) | None => {}
            }

            self.dts.text(": ");

            match field.ty.ok() {
                Some(ty) => self.write_type(ty),
                None => {
                    self.dts.text("never");
                }
            }

            match field.ty.ok() {
                Some(ty) if ty.is_nullable() => {
                    self.dts.text(" | null");
                }
                Some(_) | None => {}
            }

            self.dts.text(";\n");
        }

        self.dts.text("}\n\n");
    }

    fn process_scalar(&mut self, name: &Name<T>) {
        self.dts
            .text("export type ")
            .token(name.as_raw_token())
            .text(" = any;\n\n");
    }

    fn process_union(&mut self, name: &Name<T>) {
        self.dts
            .text("export type ")
            .token(name.as_raw_token())
            .text(" = ");

        for member in self.database.union_member_types(name.as_ref()) {
            self.dts.text("\n    | ").token(member.0.as_raw_token());
        }

        self.dts.text(";\n\n");
    }

    fn generate(mut self) -> Self {
        self.preprocess_export_directives();

        let mut tys = self.database.type_definitions().collect::<Vec<_>>();
        tys.sort_by_key(|ty| {
            ty.name()
                .ok()
                .map(|name| name.as_raw_token().source.borrow())
                .unwrap_or_default()
        });

        for ty in tys {
            self.process_type_definition(ty);
        }

        self
    }
}
