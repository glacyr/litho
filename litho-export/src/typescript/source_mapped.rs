use std::borrow::Borrow;
use std::collections::HashMap;
use std::path::Path;

use litho_language::lex::raw::RawToken;
use litho_language::lex::SourceId;
use sourcemap::{Error, SourceMapBuilder};

use super::LineIndex;

pub struct SourceMapped<'a> {
    sources: &'a HashMap<SourceId, (&'a str, LineIndex)>,
    text: String,
    builder: SourceMapBuilder,
    line: usize,
    column: usize,
}

impl<'a> SourceMapped<'a> {
    pub fn new(sources: &'a HashMap<SourceId, (&'a str, LineIndex)>) -> SourceMapped<'a> {
        SourceMapped {
            sources,
            text: String::new(),
            builder: SourceMapBuilder::new(None),
            line: 0,
            column: 0,
        }
    }

    pub fn text(&mut self, text: &str) -> &mut SourceMapped<'a> {
        self.text += text;

        let mut lines = text.split_inclusive('\n').count();
        let columns = match text.split_inclusive('\n').last().unwrap_or_default() {
            line if line.ends_with('\n') => {
                lines += 1;
                0
            }
            line => line.len(),
        };

        match lines > 1 {
            true => {
                self.column = columns;
                self.line += lines - 1;
            }
            false => self.column += columns,
        }

        self
    }

    pub fn token<T>(&mut self, token: &RawToken<T>) -> &mut SourceMapped<'a>
    where
        T: Borrow<str>,
    {
        if let Some((source, index)) = self.sources.get(&token.span.source_id) {
            let (src_line, src_column) = index.lookup(token.span.start);

            let source = self.builder.add_source(source);
            let name = self.builder.add_name(token.source.borrow());

            self.builder.add_raw(
                self.line as u32,
                self.column as u32,
                src_line as u32,
                src_column as u32,
                Some(source),
                Some(name),
            );
        }

        self.text(token.source.borrow())
    }

    pub fn write<P>(mut self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        let file_ext = path
            .extension()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        std::fs::write(
            path,
            format!("{}//# sourceMappingURL={}.map", self.text, file_name),
        )?;

        let mut js_map = Vec::new();
        self.builder.set_file(Some(file_name));
        self.builder.into_sourcemap().to_writer(&mut js_map)?;

        std::fs::write(path.with_extension(format!("{}.map", file_ext)), js_map)?;

        Ok(())
    }
}
