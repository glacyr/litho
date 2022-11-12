use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use litho_diagnostics::Diagnostic;
use litho_language::ast::{DefinitionId, Document};
use litho_language::chk::collect_errors;
use litho_language::lex::{SourceId, Span};
use litho_language::Parse;
use litho_types::Database;
use litho_validation::check;

use super::{Consumer, DepGraph, Dependency, Producer};

#[derive(Debug)]
pub struct Compiler<T>
where
    T: Eq + Hash,
{
    definition_diagnostics: HashMap<DefinitionId, Vec<Diagnostic<Span>>>,
    definition_sources: HashMap<DefinitionId, SourceId>,
    documents: HashMap<SourceId, (Arc<Document<T>>, bool)>,
    document_diagnostics: HashMap<SourceId, Vec<Diagnostic<Span>>>,
    graph: DepGraph<DefinitionId, Dependency<T>>,
    database: Database<T>,
    imports: HashMap<String, Result<SourceId, String>>,
}

impl<T> Compiler<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Compiler<T> {
        Compiler {
            definition_diagnostics: Default::default(),
            definition_sources: Default::default(),
            documents: Default::default(),
            document_diagnostics: Default::default(),
            graph: DepGraph::new(),
            database: Database::new(),
            imports: HashMap::new(),
        }
    }

    pub fn database(&self) -> &Database<T> {
        &self.database
    }

    pub fn document(&self, source_id: SourceId) -> Option<&Arc<Document<T>>> {
        self.documents.get(&source_id).map(|(doc, _)| doc)
    }
}

impl<T> Compiler<T>
where
    T: Eq + Clone + Hash + Borrow<str> + ToString,
{
    pub fn imports(&self) -> &HashMap<String, Duration> {
        self.database.imports()
    }

    pub fn update_resolved_imports(&mut self, imports: HashMap<String, Result<SourceId, String>>)
    where
        T: for<'a> From<&'a str> + for<'a> PartialEq<&'a str>,
    {
        self.imports = imports;
    }

    pub fn diagnostics(&self, source_id: SourceId) -> impl Iterator<Item = &Diagnostic<Span>> {
        let document_diagnostics = self
            .document_diagnostics
            .get(&source_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .into_iter();

        let definition_diagnostics = self
            .documents
            .get(&source_id)
            .into_iter()
            .flat_map(|document| document.0.definitions.iter())
            .flat_map(|definition| {
                self.definition_diagnostics
                    .get(&definition.id())
                    .into_iter()
                    .flatten()
            });

        document_diagnostics.chain(definition_diagnostics)
    }

    pub fn replace_document(
        &mut self,
        source_id: SourceId,
        text: &str,
        is_import: bool,
    ) -> HashSet<SourceId>
    where
        T: for<'a> From<&'a str> + for<'b> PartialEq<&'b str>,
    {
        let mut source_ids = self.remove_document(source_id);
        source_ids.extend(self.add_document(source_id, text, is_import));

        source_ids
    }

    pub fn add_document(
        &mut self,
        source_id: SourceId,
        text: &str,
        is_import: bool,
    ) -> HashSet<SourceId>
    where
        T: for<'a> From<&'a str> + for<'b> PartialEq<&'b str>,
    {
        let result = Document::parse_from_str(source_id, text).unwrap_or_default();
        let diagnostics = collect_errors(&result);

        let mut definition_ids = HashSet::new();

        for definition in result.0.definitions.iter() {
            self.definition_sources.insert(definition.id(), source_id);

            definition_ids.insert(definition.id());

            if let Some(product) = definition.product() {
                definition_ids.extend(self.graph.produce(definition.id(), product));
            }

            for dependency in definition.consumes() {
                self.graph.consume(definition.id(), dependency);
            }
        }

        self.documents
            .insert(source_id, (Arc::new(result.0), is_import));
        self.document_diagnostics.insert(source_id, diagnostics);

        self.invalidate(definition_ids)
    }

    pub fn remove_document(&mut self, source_id: SourceId) -> HashSet<SourceId> {
        let document = self.documents.remove(&source_id);

        let mut definition_ids = HashSet::new();

        for definition in document
            .as_ref()
            .map(|document| document.0.definitions.iter())
            .into_iter()
            .flatten()
        {
            self.graph.invalidate(definition.id(), &mut definition_ids);
            self.graph.remove(definition.id());
        }

        let source_ids = self.invalidate(definition_ids);

        for definition in document
            .as_ref()
            .map(|document| document.0.definitions.iter())
            .into_iter()
            .flatten()
        {
            self.definition_sources.remove(&definition.id());
        }

        source_ids
    }

    fn invalidate<I>(&mut self, definition_ids: I) -> HashSet<SourceId>
    where
        I: IntoIterator<Item = DefinitionId>,
    {
        let mut source_ids = HashSet::new();

        for definition_id in definition_ids.into_iter() {
            self.definition_diagnostics.remove(&definition_id);

            source_ids.extend(self.definition_sources.get(&definition_id).into_iter());
        }

        source_ids
    }

    pub fn rebuild(&mut self)
    where
        T: From<&'static str>,
    {
        self.database = Database::with_imports(
            self.documents.values().map(|(doc, _)| doc.as_ref()),
            &Default::default(),
        );

        for document in self.documents.values() {
            for definition in document.0.definitions.iter() {
                self.definition_diagnostics
                    .entry(definition.id())
                    .or_insert_with(|| check(definition, &self.database));
            }
        }
    }
}
