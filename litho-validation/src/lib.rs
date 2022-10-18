use std::borrow::Borrow;
use std::fmt::Display;
use std::hash::Hash;

use litho_language::ariadne::ReportKind;
use litho_language::ast::*;
use litho_language::chk::{IntoReport, LabelBuilder, ReportBuilder};
use litho_language::lex::Span;
use litho_types::Database;

mod arguments;
mod fields;
mod interfaces;
mod names;
mod types;
mod unions;

pub enum Error<'a, T> {
    UnknownNamedType {
        name: &'a T,
        span: Span,
    },
    EmptyType {
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
    ImplementsNonInterfaceType {
        name: &'a T,
        interface: &'a T,
        span: Span,
    },
    MissingInheritedInterface {
        name: &'a T,
        interface: &'a T,
        inherited: &'a T,
        span: Span,
    },
    MissingInterfaceField {
        name: &'a T,
        interface: &'a T,
        field: &'a T,
        span: Span,
    },
    MissingInterfaceFieldArgument {
        name: &'a T,
        interface: &'a T,
        field: &'a T,
        argument: &'a T,
        span: Span,
    },
    InvalidInterfaceFieldArgumentType {
        name: &'a T,
        interface: &'a T,
        field: &'a T,
        argument: &'a T,
        expected: &'a Type<T>,
        ty: &'a Type<T>,
        span: Span,
    },
    UnexpectedNonNullInterfaceFieldArgument {
        name: &'a T,
        interface: &'a T,
        field: &'a T,
        argument: &'a T,
        ty: &'a Type<T>,
        span: Span,
    },
    NonCovariantInterfaceField {
        name: &'a T,
        interface: &'a T,
        field: &'a T,
        expected: &'a Type<T>,
        ty: &'a Type<T>,
        span: Span,
    },
    SelfReferentialInterface {
        name: &'a T,
        span: Span,
    },
    MissingUnionMembers {
        name: &'a T,
        span: Span,
    },
    DuplicateUnionMember {
        name: &'a T,
        first: Span,
        second: Span,
    },
    NonObjectUnionMember {
        name: &'a T,
        span: Span,
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
            Error::EmptyType { name, span } => B::new(ReportKind::Error, span)
                .with_code("E0101")
                .with_message("Type must define one or more fields.")
                .with_label(B::LabelBuilder::new(span).with_message(format!(
                    "Type `{}` here doesn't define any fields.",
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
            Error::ImplementsNonInterfaceType {
                name,
                interface,
                span,
            } => B::new(ReportKind::Error, span)
                .with_code("E0110")
                .with_message("Type implements non-interface type.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` implements `{interface}` here, which isn't an interface.")),
                )
                .finish(),
            Error::MissingInheritedInterface {
                name,
                interface,
                inherited,
                span
            } => B::new(ReportKind::Error, span)
                .with_code("E0111")
                .with_message("Type implements interface that requires missing interface.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` implements `{interface}` here, which requires that interface `{inherited}` is also implemented, but `{inherited}` is not implemented for type `{name}`.")),
                )
                .finish(),
            Error::MissingInterfaceField {
                name,
                interface,
                field,
                span
            } => B::new(ReportKind::Error, span)
                .with_code("E0112")
                .with_message("Type implements interface that requires missing field.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` implements `{interface}` here, which defines field `{field}` but `{field}` is not defined for `{name}`.")),
                )
                .finish(),
            Error::MissingInterfaceFieldArgument {
                name,
                interface,
                field,
                argument,
                span
            } => B::new(ReportKind::Error, span)
                .with_code("E0113")
                .with_message("Type implements interface that requires field with missing argument.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` implements `{interface}` here, which defines field `{field}` with argument `{argument}` but `{field}` does not define argument `{argument}` for `{name}`.")),
                )
                .finish(),
            Error::InvalidInterfaceFieldArgumentType {
                name,
                interface,
                field,
                argument,
                expected,
                ty,
                span,
            } => B::new(ReportKind::Error, span)
                .with_code("E0114")
                .with_message("Type implements interface that requires field with argument of invalid type.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` implements `{interface}` here, which defines field `{field}` with argument `{argument}` of type `{expected}` but `{field}` defines argument `{argument}` as type `{ty}` for `{name}`.")),
                )
                .finish(),
            Error::UnexpectedNonNullInterfaceFieldArgument {
                name,
                interface,
                field,
                argument,
                ty,
                span,
            } => B::new(ReportKind::Error, span)
                .with_code("E0114")
                .with_message("Type defines non-null argument that does not appear in interface field.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` implements `{interface}` here, which defines field `{field}` with argument `{argument}` of type `{ty}` but this type may not be non-null since field `{field}` does not define an argument `{argument}` for interface `{interface}`.")),
                )
                .finish(),
            Error::NonCovariantInterfaceField {
                name,
                interface,
                field,
                expected,
                ty,
                span,
            } => B::new(ReportKind::Error, span)
                .with_code("E0115")
                .with_message("Type defines field with type that is not compatible with interface field's type.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` implements `{interface}` here, which defines field `{field}` type `{ty}` but this type is not compatible with expected type `{expected}` of field `{field}` interface `{interface}`.")),
                )
                .finish(),
            Error::SelfReferentialInterface {
                name, span
            } => B::new(ReportKind::Error, span)
                .with_code("E0116")
                .with_message("Type may not implement its own interface.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Type `{name}` attempts to implement self-referential interface here, which is not allowed.")),
                )
                .finish(),
            Error::MissingUnionMembers {
                name, span
            } => B::new(ReportKind::Error, span)
                .with_code("E0117")
                .with_message("Union type must reference at least one member type.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Union type `{name}` does not define any member types here.")),
                )
                .finish(),
            Error::DuplicateUnionMember {
                name,
                first,
                second,
            } => B::new(ReportKind::Error, second)
                .with_code("E0118")
                .with_message("Union type defines same member type twice.")
                .with_label(
                    B::LabelBuilder::new(first)
                        .with_message(format!("Union type first refers to `{name}` here ...")),
                )
                .with_label(
                    B::LabelBuilder::new(second)
                        .with_message(format!("... and later again here.")),
                )
                .finish(),
            Error::NonObjectUnionMember {
                name, span
            } => B::new(ReportKind::Error, span)
                .with_code("E0119")
                .with_message("Union type can only reference object types.")
                .with_label(
                    B::LabelBuilder::new(span)
                        .with_message(format!("Union type refers to `{name}` here, but `{name}` cannot be used as an union member type because it is not an object type.")),
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
    document.traverse(&fields::HasFields(database), &mut errors);
    document.traverse(&fields::FieldNameUniqueness(database), &mut errors);
    document.traverse(&fields::FieldsAreOutputTypes(database), &mut errors);
    document.traverse(&interfaces::ImplementsInterface(database), &mut errors);
    document.traverse(&names::ReservedNames(database), &mut errors);
    document.traverse(&types::NamedTypesExist(database), &mut errors);
    document.traverse(&unions::UnionMemberTypes(database), &mut errors);
    errors
}
