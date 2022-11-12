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

    pub fn get_mut(&mut self, source_id: &SourceId) -> Option<&mut Document> {
        self.documents.get_mut(source_id)
    }

    pub fn insert(
        &mut self,
        id: SourceId,
        url: Url,
        version: Option<i32>,
        internal: bool,
        text: String,
    ) -> &Document {
        self.documents
            .insert(id, Document::new(id, url, version, internal, text.as_ref()));

        self.get(&id).unwrap()
    }

    pub fn update<F>(&mut self, id: SourceId, url: Url, version: Option<i32>, apply: F) -> String
    where
        F: FnOnce(String) -> String,
    {
        let document = self.documents.get(&id).unwrap();
        let text = apply(document.text().to_owned());
        self.documents.insert(
            id,
            Document::new(id, url, version, document.is_internal(), text.as_ref()),
        );

        text
    }

    pub fn remove(&mut self, id: &SourceId) -> Option<Document> {
        self.documents.remove(id)
    }
}
