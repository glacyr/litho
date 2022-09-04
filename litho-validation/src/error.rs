use std::borrow::Cow;
use std::fmt::Display;

use graphql_parser::query::{Field, FragmentDefinition, OperationDefinition, Text, Type};

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

    /// Caused by violation of [`SingleRootField`](5.2.3.1 Single Root Field)
    MultipleSubscriptionRoots {
        first_name: &'a str,
        first_span: Span,
        second_name: &'a str,
        second_span: Span,
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

    /// Caused by violation of [`ArgumentNames`](5.4.1 Argument Names).
    UndefinedDirectiveArgument {
        directive_name: &'a T::Value,
        directive_span: Span,
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

    /// Caused by violation of [`FragmentSpreadTargetDefined`](5.5.2.1 Fragment Spread Target Defined).
    UndefinedFragment {
        fragment_name: &'a T::Value,
        span: Span,
    },

    /// Caused by violation of [`FragmentSpreadsMustNotFormCycles`](5.5.2.2 Fragment Spreads Must Not Form Cycles).
    CyclicFragmentSpread {
        fragment_name: &'a T::Value,
        span: Span,
    },

    /// Caused by violation of [`FragmentSpreadIsPossible`](5.5.2.3 Fragment Spread Is Possible)
    ImpossibleFragmentSpread {
        fragment_name: &'a str,
        fragment_type: &'a str,
        fragment_span: Span,
        parent_type: &'a str,
        parent_span: Span,
    },

    /// Caused by violation of [`FragmentSpreadIsPossible`](5.5.2.3 Fragment Spread Is Possible)
    ImpossibleInlineFragment {
        fragment_type: &'a str,
        fragment_span: Span,
        parent_type: &'a str,
        parent_span: Span,
    },

    /// Caused by violation of [`DirectivesAreDefined`](5.7.1 Directives Are Defined)
    UndefinedDirective { name: &'a str, span: Span },

    /// Caused by violation of [`DirectivesAreInValidLocations`](5.7.2 Directives Are In Valid Locations)
    InvalidDirectiveLocation { name: &'a str, span: Span },

    /// Caused by violation of [`ValuesOfCorrectType`](5.6.1 Values Of Correct Type)
    IncorrectValueType {
        name: Cow<'a, str>,
        span: Span,
        expected: &'a Type<'b, T>,
        actual: ValueType,
    },

    /// Caused by violation of [`InputObjectFieldNames`](5.6.2 Input Object Field Names)
    UndefinedInputObjectField {
        name: Cow<'a, str>,
        span: Span,
        ty: &'a Type<'b, T>,
        field: &'a str,
    },

    /// Caused by violation of [`InputObjectRequiredFields`](5.6.4 Input Object Required Fields)
    MissingRequiredInputObjectField {
        name: Cow<'a, str>,
        span: Span,
        ty: &'a Type<'b, T>,
        field: &'a str,
    },

    /// Caused by violation of [`VariableUniqueness`](5.8.1 Variable Uniqueness)
    DuplicateVariableName {
        name: &'a str,
        first: Span,
        second: Span,
    },

    /// Caused by violation of [`VariablesAreInputTypes`](5.8.2 Variables Are Input Types)
    NonInputVariable {
        name: &'a str,
        ty: &'a str,
        span: Span,
    },

    /// Caused by violation of [`AllVariableUsesDefined`](5.8.3 All Variable Uses Defined)
    UndefinedVariable { name: &'a str, span: Span },

    /// Caused by violation of [`AllVariablesUsed`](5.8.4 All Variables Used)
    UnusedVariable { name: &'a str, span: Span },
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

            Error::MultipleSubscriptionRoots { first_name, first_span, second_name, second_span } => {
                Diagnostic::new("5.2.3.1 Single Root Field")
                .message("Subscriptions should have exactly one field.")
                .label(format!("Subscriptions should have exactly one field but first field `{}` is referenced here ...", first_name), first_span)
                .label(format!("... and second field `{}` is referenced here.", second_name), second_span)
            }

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
			Error::UndefinedDirectiveArgument { directive_name, directive_span, name } => {
				Diagnostic::new("5.4.1 Argument Names")
				.message(format!("Given argument `{}` is not defined for directive `{}`.", name, directive_name))
				.label(format!("Argument `{}` does not exist on directive `{}`.", name, directive_name), directive_span)
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
            Error::UndefinedFragment { fragment_name, span } => {
                Diagnostic::new("5.5.2.1 Fragment Spread Target Defined")
                .message(format!("Fragment `{}` is not defined.", fragment_name))
                .label(format!("Fragment `{}` is referenced here but never defined.", fragment_name), span)
            }
            Error::CyclicFragmentSpread { fragment_name, span } => {
                Diagnostic::new("5.5.2.2 Fragment Spreads Must Not Form Cycles")
                .message(format!("Fragment `{}` has already been spread.", fragment_name))
                .label(format!("Spreading fragment `{}` again here forms a cycle.", fragment_name), span)
            }

            Error::ImpossibleFragmentSpread { fragment_name, fragment_type, fragment_span, parent_type, parent_span } => {
                Diagnostic::new("5.5.2.3 Fragment Spread Is Possible")
                .message(format!("Fragment `{}` can only be applied to type `{}`.", fragment_name, fragment_type))
                .label(format!("Fragment `{}` can only be applied to type `{}` ...", fragment_name, fragment_type), fragment_span)
                .label(format!("... but is used on selection of type `{}` here.", parent_type), parent_span)
            }

            Error::ImpossibleInlineFragment { fragment_type, fragment_span, parent_type, parent_span } => {
                Diagnostic::new("5.5.2.3 Fragment Spread Is Possible")
                .message(format!("Fragment is applied to unrelated type `{}`.", fragment_type))
                .label(format!("Fragment is used on selection of type `{}` here ...", parent_type), parent_span)
                .label(format!("... but applies to type `{}` here.", fragment_type), fragment_span)
            }

            Error::UndefinedDirective { name, span } => {
                Diagnostic::new("5.7.1 Directives Are Defined")
                .message(format!("Directive `{}` does not exist.", name))
                .label(format!("Directive `{}` is referenced here but does not exist.", name), span)
            }

            Error::InvalidDirectiveLocation { name, span } => {
                Diagnostic::new("5.7.2 Directives Are In Valid Locations")
                .message(format!("Directive `{}` cannot be used in this location.", name))
                .label(format!("Directive `{}` is used here but cannot be used in this location.", name), span)
            }

            Error::IncorrectValueType { name, span, expected, actual } => {
                Diagnostic::new("5.6.1 Values Of Correct Type")
                .message(format!("Value provided for `{}` has incorrect type.", name))
                .label(format!("Value provided for `{}` here is type `{}` but expected `{}`.", name, actual.as_str(), expected), span)
            }

            Error::UndefinedInputObjectField { name, span, ty, field } => {
                Diagnostic::new("5.6.2 Input Object Field Names")
                .message(format!("Field `{}` does not exist for input type `{}`.", field, ty))
                .label(format!("Input `{}` resolves to type `{}` here but does not have field `{}`.", name, ty, field), span)
            }

            Error::MissingRequiredInputObjectField { name, span, ty, field } => {
                Diagnostic::new("5.6.4 Input Object Required Fields")
                .message(format!("Field `{}` is required for input type `{}`.", field, ty))
                .label(format!("Input `{}` resolves to type `{}` here but requires field `{}`.", name, ty, field), span)
            }

            Error::DuplicateVariableName { name, first, second } => {
                Diagnostic::new("5.8.1 Variable Uniqueness")
                .message(format!("Variable `{}` is defined twice.", name))
                .label(format!("Variable `{}` is first defined here ...", name), first)
                .label("... and later defined again here.", second)
            }

            Error::NonInputVariable { name, ty, span } => {
                Diagnostic::new("5.8.2 Variables Are Input Types")
                .message(format!("Variable `{}` isn't input type.", name))
                .label(format!("Variable `{}` is defined here but `{}` is not an input type.", name, ty), span)
            }

            Error::UndefinedVariable { name, span } => {
                Diagnostic::new("5.8.3 All Variable Uses Defined")
                .message(format!("Variable `{}` is not defined.", name))
                .label(format!("Variable `{}` is used here but not defined anywhere.", name), span)
            }

            Error::UnusedVariable { name, span } => {
                Diagnostic::new("5.8.4 All Variables Used")
                .message(format!("Variable `{}` is never used.", name))
                .label(format!("Variable `{}` is defined here but never used.", name), span)
            }
        }
    }
}
