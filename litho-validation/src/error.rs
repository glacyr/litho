use std::fmt::Display;

use graphql_parser::query::{Field, FragmentDefinition, OperationDefinition, Text};

use crate::diagnostics::{Diagnostic, IntoDiagnostic};
use crate::extensions::*;

#[derive(Debug)]
pub enum Error<'a, 'b, T>
where
    T: Text<'b>,
{
    /// Caused by violation of [`OperationNameUniqueness`](5.2.1.1 Operation Name Uniqueness).
    DuplicateOperationName(
        &'a OperationDefinition<'b, T>,
        &'a OperationDefinition<'b, T>,
    ),

    /// Caused by violation of [`LoneAnonymousOperation`](5.2.2.1 Lone Anonymous Operation).
    MixedAnonymousOperation {
        anonymous: Vec<&'a OperationDefinition<'b, T>>,
        named: Vec<&'a OperationDefinition<'b, T>>,
    },

    /// Caused by violation of [`FieldSelections`](5.3.1 Field Selections).
    UndefinedField {
        field_name: Option<&'a str>,
        parent_span: Span,
        ty: &'a T::Value,
        field: &'a Field<'b, T>,
    },

    /// Caused by violation of [`LeafFieldSelections`](5.3.3 Leaf Field Selections).
    UnexpectedSubselection {
        field_name: Option<&'a str>,
        parent_span: Span,
        ty: String,
        span: Span,
    },

    /// Caused by violation of [`LeafFieldSelections`](5.3.3 Leaf Field Selections).
    MissingSubselection {
        field_name: Option<&'a str>,
        parent_span: Span,
        ty: &'a T::Value,
    },

    /// Caused by violation of [`ArgumentNames`](5.4.1 Argument Names).
    UndefinedArgument {
        field_name: &'a T::Value,
        field_span: Span,
        ty: T::Value,
        name: &'a T::Value,
    },

    /// Caused by violation of [`ArgumentUniqueness`](5.4.2 Argument Uniqueness).
    DuplicateArgumentName {
        field_name: &'a T::Value,
        field_span: Span,
        ty: T::Value,
        name: &'a str,
    },

    /// Caused by violation of [`RequiredArguments`](5.4.2.1 Required Arguments).
    MissingRequiredArgument {
        field_name: &'a T::Value,
        field_span: Span,
        ty: T::Value,
        name: &'a T::Value,
    },

    /// Caused by violation of [`FragmentNameUniqueness`](5.5.1.1 Fragment Name Uniqueness).
    DuplicateFragmentName(&'a FragmentDefinition<'b, T>, &'a FragmentDefinition<'b, T>),

    /// Caused by violation of [`FragmentSpreadTypeExistence`](5.5.1.2 Fragment Spread Type Existence).
    UndefinedFragmentTarget {
        fragment_name: Option<&'a T::Value>,
        span: Span,
        name: &'a T::Value,
    },

    /// Caused by violation of [`FragmentsOnCompositeTypes`](5.5.1.3 Fragments on Composite Types).
    NonCompositeFragmentTarget {
        fragment_name: Option<&'a T::Value>,
        span: Span,
        name: &'a T::Value,
    },

    /// Caused by violation of [`FragmentsMustBeUsed`](5.5.1.4 Fragments Must Be Used).
    UnusedFragment {
        fragment_name: &'a T::Value,
        span: Span,
    },
}

impl<'a, 'b, T> IntoDiagnostic for Error<'a, 'b, T>
where
    T: Text<'b>,
    T::Value: Display,
{
    fn into_diagnostic(self) -> Diagnostic {
        match self {
            Error::DuplicateOperationName(first, second) => {
                Diagnostic::new("5.2.1.1 Operation Name Uniqueness")
                    .message(match first.name() {
                        Some(name) => format!("Operation `{}` is defined multiple times.", name),
                        None => "Your document contains multiple anonymous operations.".to_owned(),
                    })
                    .label(
                        match first.name() {
                            Some(name) => format!("`{}` first defined here ...", name),
                            None => "first defined here ...".to_owned(),
                        },
                        first.span(),
                    )
                    .label("... and again defined here.", second.span())
            }
            Error::MixedAnonymousOperation { anonymous, named } => Diagnostic::new(
                "5.2.2.1 Lone Anonymous Operation",
            )
            .message("Documents with an anonymous operation cannot contain any other operations.")
            .label("This defines the anonymous operation.", anonymous[0].span())
            .label(
                "This cannot coexist with the anonymous operation.",
                anonymous
                    .iter()
                    .skip(1)
                    .chain(named.iter())
                    .next()
                    .unwrap()
                    .span(),
            ),
            Error::UndefinedField {
                field_name,
                parent_span,
                ty,
                field,
            } => Diagnostic::new("5.3.1 Field Selections")
                .message(format!(
                    "Type `{}` does not have field `{}`.",
                    ty, field.name
                ))
                .label(
                    match field_name {
                        Some(field_name) => {
                            format!("Field `{}` resolves to `{}` here ...", field_name, ty)
                        }
                        None => format!("Root resolves to `{}` here ...", ty),
                    },
                    parent_span,
                )
                .label(
                    format!(
                        "... but type `{}` does not have field `{}`.",
                        ty, field.name
                    ),
                    field.span(),
                ),
            Error::UnexpectedSubselection {
                field_name,
                parent_span,
                ty,
                span,
            } => Diagnostic::new("5.3.3 Leaf Field Selections")
			.message(format!("Subselection given for scalar type `{}`.", ty))
                .label(
                    match field_name {
                        Some(field_name) => {
                            format!("Field `{}` resolves to `{}` here ...", field_name, ty)
                        }
                        None => format!("Root resolves to `{}` here ...", ty),
                    },
                    parent_span,
                )
                .label(
                    format!(
                        "... but scalar type `{}` can not have a subselection.",
                        ty,
                    ),
                    span,
                ),
            Error::MissingSubselection {
                field_name,
                parent_span,
                ty,
            } => Diagnostic::new("5.3.3 Leaf Field Selections")
			.message(format!("Subselection missing for composite type `{}`.", ty))
			.label(
				match field_name {
					Some(field_name) => {
						format!("Field `{}` resolves to a composite type `{}` here but is missing a subselection.", field_name, ty)
					}
					None => format!("Root resolves to a composite type `{}` here but is missing a subselection.", ty),
				},
				parent_span,
			),
			Error::UndefinedArgument { field_name, field_span, ty, name } => {
				Diagnostic::new("5.4.1 Argument Names")
				.message(format!("Given argument `{}` is not defined for field `{}` of type `{}`.", name, field_name, ty))
				.label(format!("Argument `{}` does not exist on field `{}` of type `{}`.", name, field_name, ty), field_span)
			}
			Error::DuplicateArgumentName { field_name, field_span, ty, name } => {
				Diagnostic::new("5.4.1 Argument Names")
				.message(format!("Given argument `{}` is given more than once to field `{}` of type `{}`.", name, field_name, ty))
				.label(format!("Argument `{}` is given more than once to field `{}` of type `{}`.", name, field_name, ty), field_span)
			}
            Error::MissingRequiredArgument { field_name, field_span, ty, name } => {
                Diagnostic::new("5.4.2.1 Required Arguments")
                .message(format!("Required argument `{}` is missing for field `{}` of type `{}`.", name, field_name, ty))
                .label(format!("Argument `{}` is required but missing for field `{}` of type `{}`.", name, field_name, ty), field_span)
            }
            Error::DuplicateFragmentName(first, second) => {
                Diagnostic::new("5.5.1.1 Fragment Name Uniqueness")
                .message(format!("Fragment `{}` is defined multiple times.", first.name))
                .label(
                        format!("`{}` first defined here ...", first.name),
                    first.span(),
                )
                .label("... and again defined here.", second.span())
            }
            Error::UndefinedFragmentTarget { fragment_name, span, name } => {
                Diagnostic::new("5.5.1.2 Fragment Spread Type Existence")
                .message(match fragment_name {
                    Some(fragment_name) => format!("Fragment `{}` targets non-existent type `{}`.", fragment_name, name),
                    None => format!("Inline fragment targets non-existent type `{}`.", name)
            })
                .label(match fragment_name {
                    Some(fragment_name) => format!("Fragment `{}` targets type `{}` here, but it does not exist.", fragment_name, name),
                    None => format!("Inline fragment targets type `{}` here, but it does not exist.", name)
                }, span)
            }
            Error::NonCompositeFragmentTarget { fragment_name, span, name } => {
                Diagnostic::new("5.5.1.3 Fragments On Composite Types")
                .message(match fragment_name {
                    Some(fragment_name) => format!("Fragment `{}` targets non-composite type `{}`.", fragment_name, name),
                    None => format!("Inline fragment targets non-composite type `{}`.", name)
            })
                .label(match fragment_name {
                    Some(fragment_name) => format!("Fragment `{}` targets type `{}` here, but it is not a composite type (union, interface or object).", fragment_name, name),
                    None => format!("Inline fragment targets type `{}` here, but it is not a composite type (union, interface or object).", name)
                }, span)
            }
            Error::UnusedFragment { fragment_name, span } => {
                Diagnostic::new("5.5.1.4 Fragments Must Be Used")
                .message(format!("Fragment `{}` is never used.", fragment_name))
                .label(format!("Fragment `{}` is defined here but never used.", fragment_name), span)
            }
        }
    }
}
