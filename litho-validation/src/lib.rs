use std::borrow::Borrow;
use std::fmt::Display;
use std::hash::Hash;

use litho_language::ariadne::ReportKind;
use litho_language::ast::*;
use litho_language::chk::{IntoReport, LabelBuilder, ReportBuilder};
use litho_language::lex::Span;
use litho_types::Database;

mod arguments;
mod names;
mod objects;
mod types;

pub enum Error<'a, T> {
    UnknownNamedType {
        name: &'a T,
        span: Span,
    },
    EmptyObjectType {
        name: &'a T,
        span: Span,
    },
    MissingFieldsDefinition {
        name: &'a T,
        span: Span,
    },
    DuplicateFieldName {
        name: &'a T,
        first: Span,
        second: Span,
    },
    ReservedFieldName {
        name: &'a T,
        span: Span,
    },
    FieldNotOutputType {
        name: &'a T,
        span: Span,
    },
    DuplicateArgumentName {
        name: &'a T,
        first: Span,
        second: Span,
    },
    ReservedInputValueName {
        name: &'a T,
        span: Span,
    },
    InputValueNotInputType {
        name: &'a T,
        span: Span,
    },
    DuplicateImplementsInterface {
        name: &'a T,
        first: Span,
        second: Span,
    },
}

impl<'a, T> IntoReport for Error<'a, T>
where
    T: Display,
{
    fn into_report<B>(self) -> B::Report
    where
        B: ReportBuilder,
    {
        match self {
            Error::UnknownNamedType { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0100")
                .with_message("Type referred to here does not exist.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{}` does not exist.", name)),
                )
                .finish(),
            Error::EmptyObjectType { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0101")
                .with_message("Object type must define one or more fields.")
                .with_label(B::LabelBuilder::new(span).with_message(format!(
                    "Object type `{}` here doesn't define any fields.",
                    name
                )))
                .finish(),
            Error::MissingFieldsDefinition { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0102")
                .with_message("Object type is missing fields definition.")
                .with_label(B::LabelBuilder::new(span).with_message(format!(
                    "Consider turning `type {name}` here into `scalar {name}`.",
                )))
                .finish(),
            Error::DuplicateFieldName {
                name,
                first,
                second,
            } => B::new(ReportKind::Error, second)
                .with_code("E0103")
                .with_message("Duplicate field name.")
                .with_label(
                    B::LabelBuilder::new(first)
                        .with_message(format!("Field `{name}` is first defined here ...",)),
                )
                .with_label(
                    B::LabelBuilder::new(second)
                        .with_message(format!("... and later defined again here.",)),
                )
                .finish(),
            Error::ReservedFieldName { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0104")
                .with_message("Reserved field name.")
                .with_label(B::LabelBuilder::new(span).with_message(format!(
                    "Field `{name}` is defined here but names starting with `__` are reserved.",
                )))
                .finish(),
            Error::FieldNotOutputType { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0105")
                .with_message("Type referred to here cannot be used as an output type.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{}` here is an input type only.", name)),
                )
                .finish(),
            Error::DuplicateArgumentName {
                name,
                first,
                second,
            } => B::new(ReportKind::Error, second)
                .with_code("E0106")
                .with_message("Duplicate argument name.")
                .with_label(
                    B::LabelBuilder::new(first)
                        .with_message(format!("Argument `{name}` is first defined here ...",)),
                )
                .with_label(
                    B::LabelBuilder::new(second)
                        .with_message(format!("... and later defined again here.",)),
                )
                .finish(),
            Error::ReservedInputValueName { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0107")
                .with_message("Reserved input value name.")
                .with_label(B::LabelBuilder::new(span).with_message(format!(
                    "Input value `{name}` is defined here but names starting with `__` are reserved.",
                )))
                .finish(),
            Error::InputValueNotInputType { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0108")
                .with_message("Type referred to here cannot be used as an input type.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{}` here is an output type only.", name)),
                )
                .finish(),
            Error::DuplicateImplementsInterface {
                name,
                first,
                second,
            } => B::new(ReportKind::Error, second)
                .with_code("E0109")
                .with_message("Type implements same interface twice.")
                .with_label(
                    B::LabelBuilder::new(first)
                        .with_message(format!("Type implements interface `{name}` here ...",)),
                )
                .with_label(
                    B::LabelBuilder::new(second)
                        .with_message(format!("... and later again here.",)),
                )
                .finish(),
        }
    }
}

pub fn check<'a, T>(
    document: &'a Document<T>,
    database: &'a Database<T>,
) -> Vec<impl IntoReport + 'a>
where
    T: Eq + Hash + Display + Borrow<str>,
{
    let mut errors = vec![];
    document.traverse(&arguments::ArgumentNameUniqueness(database), &mut errors);
    document.traverse(&arguments::ArgumentsAreInputTypes(database), &mut errors);
    document.traverse(&names::ReservedNames(database), &mut errors);
    document.traverse(&objects::ObjectHasFields(database), &mut errors);
    document.traverse(&objects::FieldNameUniqueness(database), &mut errors);
    document.traverse(&objects::FieldsAreOutputTypes(database), &mut errors);
    document.traverse(&objects::ObjectImplementsInterfaces(database), &mut errors);
    document.traverse(&types::NamedTypesExist(database), &mut errors);
    errors
}
