use std::fs::{read_dir, read_to_string, write};
use std::path::Path;

use ariadne::{Cache, Report, Source};
use litho_language::chk::collect_errors;
use litho_language::lex::{SourceId, Span};
use litho_language::{Document, Parse};
use litho_types::Database;
use litho_validation::check;

struct SingleSource(Source);

impl Cache<SourceId> for SingleSource {
    fn fetch(&mut self, _id: &SourceId) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(&self.0)
    }

    fn display<'a>(&self, _id: &'a SourceId) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("graphql"))
    }
}

pub fn main() {
    for entry in read_dir("tests").unwrap() {
        let entry = entry.unwrap();
        let source = read_to_string(entry.path()).unwrap();

        let builtins = Document::<String>::parse_from_str(
            Default::default(),
            r#"
        scalar Boolean
        "#,
        )
        .unwrap();

        let ast = Document::<String>::parse_from_str(Default::default(), &source).unwrap();
        let mut errors = collect_errors(&ast);
        let mut database = Database::new();
        database.index(&builtins.0);
        database.index(&ast.0);
        errors.extend(check(&ast.0, &database));

        let output = errors
            .into_iter()
            .map(|err| err.into())
            .map(|report: Report<Span>| {
                let mut output = strip_ansi_escapes::Writer::new(Vec::new());
                report
                    .write(SingleSource(Source::from(&source)), &mut output)
                    .unwrap();
                String::from_utf8(output.into_inner().unwrap()).unwrap()
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        write(
            Path::new("snapshots").join(entry.path().file_stem().unwrap()),
            output,
        )
        .unwrap();
    }
}
