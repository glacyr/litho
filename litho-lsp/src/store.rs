use std::collections::HashMap;

use tower_lsp::lsp_types::Url;

use super::Document;

#[derive(Debug)]
pub struct Store {
    documents: HashMap<String, Document>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            documents: HashMap::new(),
        }
    }

    pub fn get(&self, url: &Url) -> Option<&Document> {
        self.documents.get(url.as_str())
    }

    pub fn insert(&mut self, url: Url, version: i32, text: String) -> &Document {
        self.documents.insert(
            url.to_string(),
            Document::new(url.clone(), version, text.as_ref()),
        );

        self.get(&url).unwrap()
    }

    pub fn update<F>(&mut self, url: Url, version: i32, apply: F) -> &Document
    where
        F: FnOnce(String) -> String,
    {
        let document = self.documents.get(url.as_str()).unwrap();
        let text = apply(document.text().to_owned());
        self.documents.insert(
            url.to_string(),
            Document::new(url.clone(), version, text.as_ref()),
        );

        self.get(&url).unwrap()
    }
}
