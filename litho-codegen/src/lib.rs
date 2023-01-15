use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;

use litho_language::lex::SourceId;
use litho_types::Database;

mod typescript;

use typescript::{codegen_typescript, TypescriptError};

#[derive(Debug)]
pub enum CodegenError {
    UnrecognizedExtension,
    Typescript(TypescriptError),
}

impl From<TypescriptError> for CodegenError {
    fn from(value: TypescriptError) -> Self {
        CodegenError::Typescript(value)
    }
}

pub fn codegen<T, P>(
    database: &Database<T>,
    source_map: HashMap<SourceId, (&str, &str)>,
    path: P,
) -> Result<(), CodegenError>
where
    T: Eq + Hash + Borrow<str>,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("js" | "ts") => codegen_typescript(database, source_map, path).map_err(Into::into),
        _ => Err(CodegenError::UnrecognizedExtension),
    }
}
