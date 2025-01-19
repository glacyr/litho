use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::{ContextValue, Definition, DefinitionId, Document};
use litho_language::chk::collect_errors;
use litho_language::lex::{SourceId, Span, Token};
use litho_types::{Database, Import};
use litho_validation::check;

use super::{Consumer, DepGraph, Dependency, Producer};

#[derive(Debug)]
pub struct Compiler<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    definition_diagnostics: HashMap<DefinitionId, Vec<Diagnostic<Span>>>,
    definition_sources: HashMap<DefinitionId, SourceId>,
    documents: HashMap<SourceId, (Arc<Document<'a, T>>, bool)>,
    document_diagnostics: HashMap<SourceId, Vec<Diagnostic<Span>>>,
    graph: DepGraph<DefinitionId, Dependency<T>>,
    database: Database<'a, T>,
    imports: HashMap<String, Result<SourceId, String>>,
}

impl<'a, T> Compiler<'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fn new() -> Compiler<'a, T> {
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

    pub fn database(&self) -> &Database<'a, T> {
        &self.database
    }

    pub fn document(&self, source_id: SourceId) -> Option<&Arc<Document<'a, T>>> {
        self.documents.get(&source_id).map(|(doc, _)| doc)
    }
}

impl<'a, T> Compiler<'a, T>
where
    T: ContextValue<'a> + Eq + Hash + Borrow<str> + ToString,
{
    pub fn imports(&self) -> &HashMap<String, Import> {
        self.database.imports()
    }

    pub fn update_resolved_imports(&mut self, imports: HashMap<String, Result<SourceId, String>>)
    where
        T: for<'b> From<&'b str> + for<'b> PartialEq<&'b str>,
    {
        self.imports = imports;
    }

    pub fn diagnostics(
        &self,
        source_id: SourceId,
    ) -> impl Iterator<Item = &Diagnostic<Span>> + use<'_, 'a, T> {
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
                    .get(&Definition::id(definition))
                    .into_iter()
                    .flatten()
            });

        document_diagnostics.chain(definition_diagnostics)
    }

    pub fn replace_document(
        &mut self,
        source_id: SourceId,
        document: (Document<'a, T>, Vec<Token<'a, T>>),
        is_import: bool,
    ) -> HashSet<SourceId>
    where
        T: for<'b> From<&'b str> + for<'b> PartialEq<&'b str>,
    {
        let mut source_ids = self.remove_document(source_id);
        source_ids.extend(self.add_document(source_id, document, is_import));

        source_ids
    }

    pub fn add_document(
        &mut self,
        source_id: SourceId,
        document: (Document<'a, T>, Vec<Token<'a, T>>),
        is_import: bool,
    ) -> HashSet<SourceId>
    where
        T: for<'b> From<&'b str> + for<'b> PartialEq<&'b str>,
    {
        let diagnostics = collect_errors(&document);

        let mut definition_ids = HashSet::new();

        for definition in document.0.definitions.iter() {
            let definition_id = Definition::id(definition);

            self.definition_sources.insert(definition_id, source_id);

            definition_ids.insert(definition_id);

            if let Some(product) = definition.product() {
                definition_ids.extend(self.graph.produce(definition_id, product));
            }

            for dependency in definition.consumes() {
                self.graph.consume(definition_id, dependency);
            }
        }

        self.documents
            .insert(source_id, (Arc::new(document.0), is_import));
        self.document_diagnostics.insert(source_id, diagnostics);

        let mut set = self.invalidate(definition_ids);
        set.insert(source_id);
        set
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
            self.graph
                .invalidate(Definition::id(definition), &mut definition_ids);
            self.graph.remove(Definition::id(definition));
        }

        let source_ids = self.invalidate(definition_ids);

        for definition in document
            .as_ref()
            .map(|document| document.0.definitions.iter())
            .into_iter()
            .flatten()
        {
            self.definition_sources.remove(&Definition::id(definition));
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
        T: From<&'a str>,
    {
        self.database = Database::with_imports(
            self.documents.values().map(|(doc, _)| doc.as_ref()),
            &Default::default(),
        );

        for document in self.documents.values() {
            for definition in document.0.definitions.iter() {
                self.definition_diagnostics
                    .entry(Definition::id(definition))
                    .or_insert_with(|| check(definition, &self.database));
            }
        }
    }
}
