use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;

use litho_language::lex::SourceId;
use litho_types::Database;
use sourcemap::Error;

mod generator;
mod line_index;
mod source_mapped;

use generator::Generator;
use line_index::LineIndex;
use source_mapped::SourceMapped;

#[derive(Debug)]
pub enum TypescriptError {
    SourceMap(Error),
}

impl From<Error> for TypescriptError {
    fn from(value: Error) -> Self {
        TypescriptError::SourceMap(value)
    }
}

pub fn export_typescript<T>(
    database: &Database<T>,
    source_map: HashMap<SourceId, (&str, &str)>,
    path: &Path,
) -> Result<(), TypescriptError>
where
    T: Eq + Hash + Borrow<str>,
{
    let source_map = source_map
        .into_iter()
        .map(|(id, (path, text))| (id, (path, LineIndex::new(text))))
        .collect();

    let generator = Generator::new(database, &source_map);

    generator.js.write(path.with_extension("js"))?;
    generator.dts.write(path.with_extension("d.ts"))?;

    Ok(())
}
