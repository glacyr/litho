use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{metadata, read_to_string};
use std::iter::once;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use ariadne::{Cache, Source};
use glob::glob;
use litho_compiler::{builtins, Compiler};
use litho_language::ast::Document;
use litho_language::lex::{SourceId, SourceMap};
use smol_str::SmolStr;

#[derive(Default)]
pub struct Sources(HashMap<SourceId, (String, Source)>);

impl Sources {
    pub fn insert(&mut self, source_id: SourceId, path: String, source: Source) {
        self.0.insert(source_id, (path, source));
    }
}

impl Cache<SourceId> for Sources {
    fn display<'a>(&self, id: &'a SourceId) -> Option<Box<dyn Display + 'a>> {
        self.0
            .get(id)
            .map(|(path, _)| Box::new(path.clone()) as Box<dyn Display>)
    }

    fn fetch(&mut self, id: &SourceId) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        self.0.get(id).map(|(_, text)| Ok(text)).unwrap()
    }
}

pub fn find(path: &str, results: &mut impl Extend<String>) {
    if path.starts_with("http://") || path.starts_with("https://") {
        results.extend(once(path.to_owned()));
        return;
    }

    if AsRef::<Path>::as_ref(path).is_dir() {
        return find(&format!("{}/**/*.graphql", path), results);
    }

    results.extend(
        glob(&path)
            .into_iter()
            .flatten()
            .map(IntoIterator::into_iter)
            .flatten()
            .map(|path| path.to_string_lossy().into_owned()),
    );
}

pub struct Workspace {
    compiler: Compiler<SmolStr>,
    texts: HashMap<SourceId, String>,
    source_map: SourceMap<String>,
    files: HashMap<SourceId, (String, std::io::Result<SystemTime>)>,
}

pub struct File<'a> {
    pub source_id: SourceId,
    pub path: &'a String,
    pub text: &'a String,
    pub document: &'a Arc<Document<SmolStr>>,
    pub modified: &'a std::io::Result<SystemTime>,
}

impl Workspace {
    pub fn new<I>(iterator: I) -> Workspace
    where
        I: IntoIterator<Item = String>,
    {
        let mut compiler = Compiler::new();
        let mut source_map = SourceMap::new();
        let mut files = HashMap::new();
        let mut texts = HashMap::new();

        for (path, text) in builtins().into_iter().copied() {
            let source_id = source_map.get_or_insert(path.to_owned());
            compiler.add_document(source_id, &text, true);
            texts.insert(source_id, text.to_owned());
        }

        let mut paths = vec![];

        for input in iterator.into_iter() {
            find(&input, &mut paths);
        }

        paths.sort();

        for path in paths {
            let source_id = source_map.get_or_insert(path.clone());
            let metadata = metadata(&path);
            let modified = metadata.and_then(|metadata| metadata.modified());
            let text = read_to_string(path.clone()).unwrap();
            compiler.add_document(source_id, &text, true);
            files.insert(source_id, (text.clone(), modified));
            texts.insert(source_id, text);
        }

        compiler.rebuild();

        Workspace {
            compiler,
            source_map,
            files,
            texts,
        }
    }

    pub fn compiler(&self) -> &Compiler<SmolStr> {
        &self.compiler
    }

    pub fn to_sources(&self) -> Sources {
        let mut sources = Sources::default();

        self.source_map.iter().for_each(|(path, &source_id)| {
            if let Some(text) = self.texts.get(&source_id) {
                sources.insert(source_id, path.clone(), Source::from(&text))
            }
        });

        sources
    }

    pub fn files(&self) -> impl Iterator<Item = File> {
        self.source_map.iter().flat_map(|(path, &source_id)| {
            let (text, modified) = self.files.get(&source_id)?;
            let document = self.compiler.document(source_id)?;
            Some(File {
                source_id,
                path,
                text,
                document,
                modified,
            })
        })
    }
}
