use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use litho_compiler::Compiler;
use litho_language::lex::{SourceId, SourceMap, Span};
use litho_types::Database;
use smol_str::SmolStr;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::diagnostic::serialize_diagnostic;

use super::{url_from_path, Document, Importer, Store};

pub struct Workspace {
    client: Client,
    store: Store,
    source_map: SourceMap<Url>,
    compiler: Compiler<SmolStr>,
    invalid: HashSet<SourceId>,
    importer: Importer,
}

impl Workspace {
    pub fn new(client: Client, importer: Importer) -> Workspace {
        Workspace {
            client,
            store: Store::new(),
            source_map: SourceMap::new(),
            compiler: Compiler::new(),
            invalid: HashSet::new(),
            importer,
        }
    }

    pub async fn importer_register(&mut self, workspace: &Arc<Mutex<Workspace>>) {
        self.importer.register(Arc::downgrade(workspace)).await;
    }

    pub async fn update_imports(&mut self, imports: HashMap<String, Result<SmolStr, String>>) {
        self.compiler.update_resolved_imports(imports);
        self.rebuild().await;
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn document_by_id(&self, id: SourceId) -> Option<&Document> {
        self.store.get(&id)
    }

    pub fn document(&self, url: &Url) -> Option<&Document> {
        self.document_by_id(self.source_map.get(url)?)
    }

    pub fn diagnostics(&self, source_id: SourceId) -> impl Iterator<Item = Diagnostic> + '_ {
        self.compiler
            .diagnostics(source_id)
            .into_iter()
            .map(|diagnostic| serialize_diagnostic(diagnostic, self))
    }

    pub fn database(&self) -> &Database<SmolStr> {
        self.compiler.database()
    }

    pub async fn mutate<F, O>(&mut self, mutation: F) -> O
    where
        F: FnOnce(&mut Workspace) -> O,
    {
        let result = mutation(self);
        self.rebuild().await;
        result
    }

    pub fn populate_inflection(&mut self) {
        self.populate_file_contents(
            Url::parse("litho://inflection.graphql").unwrap(),
            None,
            true,
            include_str!("../std/introspection.graphql").to_owned(),
        )
    }

    pub fn populate_scalars(&mut self) {
        self.populate_file_contents(
            Url::parse("litho://scalars.graphql").unwrap(),
            None,
            true,
            include_str!("../std/scalars.graphql").to_owned(),
        );
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

            let url = url_from_path(entry.path())?;
            self.populate_file(url)?;
        }

        Ok(())
    }

    pub fn populate_file(&mut self, url: Url) -> Result<(), ()> {
        let path = url.to_file_path().map_err(|_| ())?;
        let mut file = File::open(path).map_err(|_| ())?;
        let mut text = String::new();

        file.read_to_string(&mut text).map_err(|_| ())?;

        self.populate_file_contents(url, None, false, text);

        Ok(())
    }

    pub fn populate_file_contents(
        &mut self,
        url: Url,
        version: Option<i32>,
        internal: bool,
        text: String,
    ) {
        let id = self.source_map.get_or_insert(url.to_owned());

        self.invalid
            .extend(self.compiler.replace_document(id, &text));

        self.store.insert(id, url, version, internal, text);
        self.store
            .get_mut(&id)
            .into_iter()
            .for_each(|doc| doc.ast = self.compiler.document(id).cloned());
    }

    pub fn update_file_contents<F>(&mut self, url: Url, version: Option<i32>, update: F)
    where
        F: FnOnce(String) -> String,
    {
        let id = self.source_map.get_or_insert(url.to_owned());

        let text = self.store.update(id, url, version, update);

        self.invalid
            .extend(self.compiler.replace_document(id, &text));

        self.store
            .get_mut(&id)
            .into_iter()
            .for_each(|doc| doc.ast = self.compiler.document(id).cloned());
    }

    pub fn refresh_file(&mut self, url: Url) -> Result<(), ()> {
        let id = self.source_map.get_or_insert(url.to_owned());

        self.invalid.extend(self.compiler.remove_document(id));
        self.store.remove(&id);
        self.populate_file(url)?;

        Ok(())
    }

    pub async fn rebuild(&mut self) {
        self.compiler.rebuild();
        self.importer.update(self.compiler.imports()).await;
        self.check_all().await;
    }

    pub fn position_to_index(source: &str, position: Position) -> usize {
        let line_offset = source
            .split_inclusive("\n")
            .take(position.line as usize)
            .fold(0, |sum, line| sum + line.len());
        line_offset + position.character as usize
    }

    pub fn index_to_position(source: &str, index: usize) -> Position {
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
            start: Workspace::index_to_position(source, span.start),
            end: Workspace::index_to_position(source, span.end),
        })
    }

    pub fn span_to_location(&self, span: Span) -> Option<Location> {
        let uri = self.source_map.get_id(&span.source_id)?.clone();

        Some(Location {
            uri,
            range: self.span_to_range(span)?,
        })
    }

    pub fn take_invalid(&mut self) -> HashSet<SourceId> {
        std::mem::take(&mut self.invalid)
    }

    pub fn apply(mut source: String, change: TextDocumentContentChangeEvent) -> String {
        match change.range {
            Some(range) => {
                let start = Workspace::position_to_index(&source, range.start);
                let end = Workspace::position_to_index(&source, range.end);
                source.replace_range(start..end, &change.text);
            }
            None => {}
        }

        source
    }

    pub async fn check_all(&mut self) {
        for id in self.take_invalid() {
            let Some(document) = self.document_by_id(id) else {
                continue
            };

            if document.is_internal() {
                continue;
            }

            self.client
                .publish_diagnostics(
                    document.url().to_owned(),
                    self.diagnostics(id).collect(),
                    document.version(),
                )
                .await;
        }
    }
}
