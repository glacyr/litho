use crate::extensions::Span;

#[derive(Debug, Default)]
pub struct Diagnostic {
    spec: &'static str,
    message: Option<String>,
    labels: Vec<DiagnosticLabel>,
}

impl Diagnostic {
    pub fn new(spec: &'static str) -> Diagnostic {
        Diagnostic {
            spec,
            ..Default::default()
        }
    }

    pub fn message(mut self, message: impl AsRef<str>) -> Self {
        self.message = Some(message.as_ref().to_owned());
        self
    }

    pub fn label(mut self, message: impl AsRef<str>, span: Span) -> Self {
        self.labels.push(DiagnosticLabel {
            message: message.as_ref().to_owned(),
            span,
        });
        self
    }
}

#[derive(Debug)]
pub struct DiagnosticLabel {
    message: String,
    span: Span,
}

pub trait IntoDiagnostic {
    fn into_diagnostic(self) -> Diagnostic;
}

pub trait Emit<T> {
    type Error;

    fn emit(&self, source: &str) -> Result<T, Self::Error>;
}

mod graphical {
    use std::fmt::{Error, Write};

    use owo_colors::OwoColorize;

    use super::{Diagnostic, Emit};

    impl Emit<String> for Diagnostic {
        type Error = Error;

        fn emit(&self, source: &str) -> Result<String, Self::Error> {
            let mut result = String::new();

            result.write_fmt(format_args!("Error: {}\n", self.spec.red()))?;
            result.write_str("\n")?;

            if let Some(message) = self.message.as_ref() {
                result.write_fmt(format_args!("  {} {}\n", "×".red(), message))?;
            }

            let indent = source.lines().count().to_string().len();

            result.write_fmt(format_args!(" {} ╭────\n", " ".repeat(indent)))?;

            source.lines().enumerate().try_for_each(|(no, line)| {
                result.write_fmt(format_args!(
                    " {}{} │ {}\n",
                    " ".repeat(indent - (no + 1).to_string().len()),
                    (no + 1).dimmed(),
                    line
                ))?;

                self.labels
                    .iter()
                    .filter(|label| label.span.0.line == no + 1)
                    .try_for_each(|label| {
                        let len = label.span.1.max(1);
                        let half = (len - 1) / 2;

                        result.write_fmt(format_args!(
                            " {} · {}{}{}{}\n",
                            " ".repeat(indent),
                            " ".repeat(label.span.0.column - 1),
                            "─".repeat(half).cyan(),
                            "┬".cyan(),
                            "─".repeat(len - 1 - half).cyan()
                        ))?;
                        result.write_fmt(format_args!(
                            " {} · {}{} {}\n",
                            " ".repeat(indent),
                            " ".repeat(label.span.0.column - 1 + half),
                            "╰──".cyan(),
                            label.message.cyan()
                        ))?;
                        result.write_fmt(format_args!(" {} ·\n", " ".repeat(indent)))?;
                        Ok(())
                    })?;

                Ok(())
            })?;

            result.write_fmt(format_args!(" {} ╰────\n", " ".repeat(indent)))?;

            Ok(result)
        }
    }
}
