use std::collections::HashMap;

use litho_language::lex::SourceId;
use tower_lsp::lsp_types::Url;

use super::Document;

#[derive(Debug)]
pub struct Store {
    documents: HashMap<SourceId, Document>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            documents: HashMap::new(),
        }
    }

    pub fn get(&self, source_id: &SourceId) -> Option<&Document> {
        self.documents.get(source_id)
    }

    pub fn docs(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }

    pub fn insert(
        &mut self,
        id: SourceId,
        url: Url,
        version: Option<i32>,
        text: String,
    ) -> &Document {
        self.documents
            .insert(id, Document::new(id, url, version, text.as_ref()));

        self.get(&id).unwrap()
    }

    pub fn update<F>(&mut self, id: SourceId, url: Url, version: Option<i32>, apply: F) -> &Document
    where
        F: FnOnce(String) -> String,
    {
        let document = self.documents.get(&id).unwrap();
        let text = apply(document.text().to_owned());
        self.documents
            .insert(id, Document::new(id, url, version, text.as_ref()));

        self.get(&id).unwrap()
    }

    pub fn remove(&mut self, id: &SourceId) -> Option<Document> {
        self.documents.remove(id)
    }
}
