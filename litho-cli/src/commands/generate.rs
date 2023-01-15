use std::env::args;
use std::fs::{metadata, write};
use std::process::ExitCode;

use ariadne::{Label, Report, ReportKind};
use litho_language::fmt::Format;
use litho_language::lex::Span;
use yansi::Paint;

use crate::Workspace;

#[derive(Default)]
pub struct Options {
    fix: bool,
    format: bool,
}

pub enum FormattingError {
    Unformatted,
    Changed,
}

pub fn generate() -> ExitCode {
    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut options = Options::default();

    let mut args = args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fix" => options.fix = true,
            "--fmt" | "--format" => options.format = true,
            "-o" | "--output" => match args.next() {
                Some(arg) => outputs.push(arg),
                None => panic!("Missing output argument."),
            },
            _ => inputs.push(arg),
        }
    }

    let workspace = Workspace::new(inputs);

    let mut files = workspace.files().collect::<Vec<_>>();
    files.sort_by_key(|file| file.path);

    let mut sources = workspace.to_sources();

    let mut success = true;

    for file in files {
        if options.format {
            let formatted = file.document.format_to_string(80);
            if &formatted != file.text {
                let formatting_error = if options.fix {
                    let modified = metadata(file.path).and_then(|metadata| metadata.modified());

                    if file.modified.as_ref().ok() != modified.as_ref().ok() {
                        Some(FormattingError::Changed)
                    } else {
                        write(file.path, formatted).unwrap();
                        None
                    }
                } else {
                    Some(FormattingError::Unformatted)
                };

                if let Some(diagnostic) = formatting_error {
                    eprintln!(
                        "{} {}\n   {}{}{}\n",
                        Paint::red("[E0000] Error:"),
                        match diagnostic {
                            FormattingError::Changed =>
                                "File has changed on disk while formatting.",
                            FormattingError::Unformatted => "File must be formatted.",
                        },
                        Paint::new("──[").dimmed(),
                        file.path,
                        Paint::new("]").dimmed(),
                    );

                    success = false;
                }
            }
        }

        for diagnostic in workspace.compiler().diagnostics(file.source_id).cloned() {
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

            success = false;
        }
    }

    match success {
        true => {
            for output in outputs {
                litho_export::export(
                    workspace.compiler().database(),
                    workspace
                        .files()
                        .map(|file| (file.source_id, (file.path.as_str(), file.text.as_str())))
                        .collect(),
                    &output,
                )
                .unwrap();
            }

            ExitCode::SUCCESS
        }
        false => ExitCode::FAILURE,
    }
}
