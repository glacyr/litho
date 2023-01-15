use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;

use litho_language::lex::SourceId;
use litho_types::Database;

mod typescript;

use typescript::{export_typescript, TypescriptError};

#[derive(Debug)]
pub enum ExportError {
    UnrecognizedExtension,
    Typescript(TypescriptError),
}

impl From<TypescriptError> for ExportError {
    fn from(value: TypescriptError) -> Self {
        ExportError::Typescript(value)
    }
}

pub fn export<T, P>(
    database: &Database<T>,
    source_map: HashMap<SourceId, (&str, &str)>,
    path: P,
) -> Result<(), ExportError>
where
    T: Eq + Hash + Borrow<str>,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("js" | "ts") => export_typescript(database, source_map, path).map_err(Into::into),
        _ => Err(ExportError::UnrecognizedExtension),
    }
}
