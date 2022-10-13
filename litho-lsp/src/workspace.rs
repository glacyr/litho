use std::fs::File;
use std::io::Read;

use glob::glob;
use litho_types::Database;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{Document, Store};

#[derive(Debug)]
pub struct Workspace {
    store: Store,
    database: Database<SmolStr>,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            store: Store::new(),
            database: Default::default(),
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
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
        let path = url.to_file_path().map_err(|_| ())?;
        let pattern = path.join("/**/*.graphql").to_str().ok_or(())?.to_owned();
        let entries = glob(&pattern).map_err(|_| ())?;

        for entry in entries {
            let entry = entry.map_err(|_| ())?;
            let url = Url::from_file_path(entry.as_path()).map_err(|_| ())?;
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
        self.store.insert(url.to_owned(), version, text);
        self.rebuild();
        self.store.get(&url).unwrap()
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
        self.store.update(url.to_owned(), version, update);
        self.rebuild();
        self.store.get(&url).unwrap()
    }

    pub fn refresh_file(&mut self, url: Url) -> Result<(), ()> {
        self.store.remove(&url);
        self.populate_file(url)?;

        Ok(())
    }

    pub fn rebuild(&mut self) {
        self.database = Database::new();

        for document in self.store.docs() {
            self.database.index(document.ast());
        }
    }
}
