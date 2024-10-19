use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures::channel::mpsc::Sender;
use futures::lock::Mutex;
use futures::SinkExt;
use litho_compiler::{builtins, Compiler};
use litho_language::lex::{SourceId, SourceMap, Span};
use litho_types::Database;
use lsp_types::*;
use smol_str::SmolStr;

use crate::diagnostic::serialize_diagnostic;

use super::{Document, Imports, ResolvedImports, Store};

pub enum WorkspaceUpdate {
    Diagnostics {
        url: Url,
        diagnostics: Vec<Diagnostic>,
        version: Option<i32>,
    },
    Imports(Imports),
}

pub struct Workspace {
    sink: Sender<WorkspaceUpdate>,
    store: Store,
    pub source_map: SourceMap<Url>,
    compiler: Compiler<SmolStr>,
    invalid: HashSet<SourceId>,
    last_imports: ResolvedImports,
    imports: HashMap<Url, SmolStr>,
}

impl Workspace {
    pub fn new(sink: Sender<WorkspaceUpdate>) -> Arc<Mutex<Workspace>> {
        Arc::new(Mutex::new(Workspace {
            sink,
            store: Store::new(),
            source_map: SourceMap::new(),
            compiler: Compiler::new(),
            invalid: HashSet::new(),
            last_imports: ResolvedImports::new(),
            imports: HashMap::new(),
        }))
    }

    pub async fn update_imports(&mut self, imports: ResolvedImports) {
        if self.last_imports == imports {
            return;
        }

        self.last_imports = imports.clone();

        let imports = imports
            .into_iter()
            .flat_map(|(url, result)| {
                Some((
                    Url::parse_with_params(
                        "litho://import.litho.dev/imported.graphql",
                        &[("url", &url)],
                    )
                    .ok()?,
                    (url, result),
                ))
            })
            .collect::<HashMap<_, _>>();

        self.imports
            .keys()
            .filter(|url| !imports.contains_key(url))
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|url| {
                self.imports.remove(&url);
                self.remove_file(&url);
            });

        let imports = imports
            .into_iter()
            .map(|(url, (source, result))| match result {
                Ok(text) if self.imports.get(&url) == Some(&text) => {
                    (source, Ok(self.source_map.get_or_insert(url)))
                }
                Ok(text) => {
                    self.populate_file_contents(url.clone(), None, true, text.to_string());
                    self.imports.insert(url.clone(), text);
                    (source, Ok(self.source_map.get_or_insert(url)))
                }
                Err(error) => (source, Err(error)),
            })
            .collect();

        self.compiler.update_resolved_imports(imports);
        self.rebuild().await;
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

    pub fn populate_builtins(&mut self) {
        for (path, source) in builtins().into_iter().copied() {
            self.populate_file_contents(Url::parse(path).unwrap(), None, true, source.to_owned())
        }
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
            .extend(self.compiler.replace_document(id, &text, internal));

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
            .extend(self.compiler.replace_document(id, &text, false));

        self.store
            .get_mut(&id)
            .into_iter()
            .for_each(|doc| doc.ast = self.compiler.document(id).cloned());
    }

    pub fn remove_file(&mut self, url: &Url) {
        let Some(id) = self.source_map.remove(url) else {
            return;
        };

        self.invalid.extend(self.compiler.remove_document(id));
        self.store.remove(&id);
    }

    pub async fn rebuild(&mut self) {
        self.compiler.rebuild();
        let _ = self
            .sink
            .send(WorkspaceUpdate::Imports(self.compiler.imports().clone()))
            .await;
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
        let invalid = self.take_invalid();
        for id in invalid {
            let Some(document) = self.document_by_id(id) else {
                continue;
            };

            if document.is_internal() {
                continue;
            }

            let _ = self
                .sink
                .send(WorkspaceUpdate::Diagnostics {
                    url: document.url().to_owned(),
                    diagnostics: self.diagnostics(id).collect(),
                    version: document.version(),
                })
                .await;
        }
    }
}
