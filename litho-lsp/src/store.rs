use std::collections::HashMap;
use std::pin::Pin;

use tower_lsp::lsp_types::Url;

use super::Document;

#[derive(Debug)]
pub struct Store {
    documents: HashMap<String, OwnedDocument>,
}

impl Store {
    pub fn new() -> Store {
        Store {
            documents: HashMap::new(),
        }
    }

    pub fn get(&self, url: &Url) -> Option<&Document<'_>> {
        self.documents
            .get(url.as_str())
            .map(|owned| owned.document())
    }

    pub fn insert(&mut self, url: Url, version: i32, text: String) -> &Document<'_> {
        self.documents.insert(
            url.to_string(),
            OwnedDocument::new(url.clone(), version, text),
        );

        self.get(&url).unwrap()
    }

    pub fn update<F>(&mut self, url: Url, version: i32, apply: F) -> &Document<'_>
    where
        F: FnOnce(String) -> String,
    {
        let document = self.documents.get(url.as_str()).unwrap();
        let text = apply(document.text.to_string());
        self.documents.insert(
            url.to_string(),
            OwnedDocument::new(url.clone(), version, text),
        );

        self.get(&url).unwrap()
    }
}

#[derive(Debug)]
pub struct OwnedDocument {
    text: Pin<Box<str>>,
    document: Document<'static>,
}

impl OwnedDocument {
    pub fn new(url: Url, version: i32, text: String) -> OwnedDocument {
        let text: Pin<Box<str>> = text.into_boxed_str().into();
        let text_ref = unsafe { std::mem::transmute(text.as_ref()) };
        OwnedDocument {
            text,
            document: Document::new(url, version, text_ref),
        }
    }

    pub fn document<'a>(&'a self) -> &'a Document<'a> {
        &self.document
    }
}
