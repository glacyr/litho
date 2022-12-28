use std::collections::HashMap;
use std::env::args;
use std::fmt::Display;
use std::fs::read_to_string;
use std::process::ExitCode;

use ariadne::{Cache, Label, Report, ReportKind, Source};
use glob::glob;
use litho_compiler::{builtins, Compiler};
use litho_language::lex::{SourceId, SourceMap, Span};
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

pub fn main() -> ExitCode {
    let mut inputs = vec![];
    let mut args = args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" | "-v" | "version" => {
                println!(
                    "litho {} ({} {})",
                    env!("CARGO_PKG_VERSION"),
                    env!("VERGEN_GIT_SHA_SHORT"),
                    env!("VERGEN_GIT_COMMIT_DATE")
                );

                return ExitCode::SUCCESS;
            }
            arg => inputs.extend(
                glob(&arg)
                    .into_iter()
                    .flatten()
                    .map(IntoIterator::into_iter)
                    .flatten()
                    .map(|path| path.to_string_lossy().into_owned()),
            ),
        }
    }

    let mut compiler = Compiler::<SmolStr>::new();
    let mut source_map = SourceMap::new();
    let mut sources = Sources::default();

    for (path, text) in builtins().into_iter().copied() {
        let source_id = source_map.get_or_insert(path.to_owned());
        sources.insert(source_id, path.to_owned(), Source::from(&text));
        compiler.add_document(source_id, &text, true);
    }

    for input in inputs.iter() {
        let source_id = source_map.get_or_insert(input.clone());
        let text = read_to_string(input.clone()).unwrap();
        sources.insert(source_id, input.clone(), Source::from(&text));
        compiler.add_document(source_id, &text, true);
    }

    compiler.rebuild();

    let mut code = ExitCode::SUCCESS;

    for input in inputs.iter() {
        let Some(source_id) = source_map.get(&input) else {
            continue
        };

        let diagnostics = compiler.diagnostics(source_id);

        for diagnostic in diagnostics {
            let span = diagnostic.span();
            let mut builder = Report::<Span>::build(ReportKind::Error, span.source_id, span.start)
                .with_code(diagnostic.code())
                .with_message(diagnostic.message());
            builder.add_labels(
                diagnostic
                    .labels()
                    .into_iter()
                    .map(|(span, message)| Label::new(span).with_message(message)),
            );
            builder.finish().eprint(&mut sources).unwrap();
            eprintln!("");

            code = ExitCode::FAILURE;
        }
    }

    code
}
