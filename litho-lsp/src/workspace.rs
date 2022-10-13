use std::fs::File;
use std::io::Read;

use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use litho_language::lex::{SourceMap, Span};
use litho_types::Database;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;
use url_escape::percent_encoding::AsciiSet;

use super::{Document, Store};

#[derive(Debug)]
pub struct Workspace {
    store: Store,
    source_map: SourceMap<Url>,
    database: Database<SmolStr>,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            store: Store::new(),
            source_map: SourceMap::new(),
            database: Default::default(),
        }
    }

    pub fn document(&self, url: &Url) -> Option<&Document> {
        let id = self.source_map.get(url)?;
        self.store.get(&id)
    }

    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.store.docs()
    }

    pub fn database(&self) -> &Database<SmolStr> {
        &self.database
    }

    pub fn populate_inflection(&mut self) -> &Document {
        self.populate_file_contents(
            Url::parse("litho://inflection.graphql").unwrap(),
            None,
            r#"
        type Query {
            __schema: __Schema!
        }

        type __Schema {
            queryType: __Type!
        }

        type __Type {
            name: String
        }
        "#
            .to_owned(),
        )
    }

    pub fn populate_root(&mut self, url: Url) -> Result<(), ()> {
        let mut types = TypesBuilder::new();
        types.add("GraphQL", "*.graphql").unwrap();

        let path = url.to_file_path().map_err(|_| ())?;
        let walk = WalkBuilder::new(path)
            .types(types.select("GraphQL").build().unwrap())
            .follow_links(false)
            .build();

        for entry in walk {
            let entry = entry.map_err(|_| ())?;

            if entry.path().is_dir() {
                continue;
            }

            let mut url = Url::from_file_path(entry.path()).map_err(|_| ())?;

            const ESCAPE: AsciiSet = url_escape::CONTROLS
                .add(b':')
                .add(b'/')
                .add(b'?')
                .add(b'#')
                .add(b'[')
                .add(b']')
                .add(b'@')
                .add(b'!')
                .add(b'$')
                .add(b'&')
                .add(b'\'')
                .add(b'(')
                .add(b')')
                .add(b'*')
                .add(b'+')
                .add(b',')
                .add(b';')
                .add(b'=')
                .add(b' ');

            url.set_path(
                &url.path()
                    .split("/")
                    .map(|component| url_escape::encode(component, &ESCAPE))
                    .collect::<Vec<_>>()
                    .join("/"),
            );
            self.populate_file(url)?;
        }

        Ok(())
    }

    pub fn populate_file(&mut self, url: Url) -> Result<(), ()> {
        let path = url.to_file_path().map_err(|_| ())?;
        let mut file = File::open(path).map_err(|_| ())?;
        let mut text = String::new();

        file.read_to_string(&mut text).map_err(|_| ())?;

        self.populate_file_contents(url, None, text);

        Ok(())
    }

    pub fn populate_file_contents(
        &mut self,
        url: Url,
        version: Option<i32>,
        text: String,
    ) -> &Document {
        let id = self.source_map.get_or_insert(url.to_owned());

        self.store.insert(id, url, version, text);
        self.rebuild();
        self.store.get(&id).unwrap()
    }

    pub fn update_file_contents<F>(
        &mut self,
        url: Url,
        version: Option<i32>,
        update: F,
    ) -> &Document
    where
        F: FnOnce(String) -> String,
    {
        let id = self.source_map.get_or_insert(url.to_owned());

        self.store.update(id, url, version, update);
        self.rebuild();
        self.store.get(&id).unwrap()
    }

    pub fn refresh_file(&mut self, url: Url) -> Result<(), ()> {
        let id = self.source_map.get_or_insert(url.to_owned());

        self.store.remove(&id);
        self.populate_file(url)?;

        Ok(())
    }

    pub fn rebuild(&mut self) {
        self.database = Database::new();

        for document in self.store.docs() {
            self.database.index(document.ast());
        }
    }

    pub fn index_to_position(&self, source: &str, index: usize) -> Position {
        let mut line = 0;
        let mut character = 0;

        for char in source[0..index].chars() {
            if char == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }
        }

        Position { line, character }
    }

    pub fn span_to_range(&self, span: Span) -> Option<Range> {
        let source = self.store.get(&span.source_id)?.text();

        Some(Range {
            start: self.index_to_position(source, span.start),
            end: self.index_to_position(source, span.end),
        })
    }

    pub fn span_to_location(&self, span: Span) -> Option<Location> {
        let uri = self.source_map.get_id(&span.source_id)?.clone();

        Some(Location {
            uri,
            range: self.span_to_range(span)?,
        })
    }
}
