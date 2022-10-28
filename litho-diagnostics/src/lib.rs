#[macro_use]
mod dsl;

pub use dsl::DiagnosticInfo;

diagnostics! {
    E0001 => UnrecognizedTokens @ span {
        "Syntax error.",
        "Couldn't recognize these tokens here." @ span
    },
    E0002 => MissingOperationDefinitionName @ span {
        "Operation definition must have a name.",
        "Expected a name for operation definition here." @ span
    } @deprecated,
    E0003 => MissingOperationDefinitionSelectionSet @ span {
        "Operation definition must have a selection set.",
        "Expected a selection set for operation definition here." @ span
    },
    E0004 => MissingSelectionSetClosingBrace @ second {
        "Selection set is missing closing curly brace.",
        "This `{{` here ..." @ first,
        "... should have a matching `}}` here." @ second
    },
    E0005 => MissingFieldName @ span {
        "Field must have a name.",
        "Missing name of this field here." @ span
    },
    E0006 => MissingArgumentsClosingParentheses @ second {
        "Arguments are missing closing parenthesis.",
        "This `(` here ..." @ first,
        "... should have a matching `)` here." @ second
    },
    E0007 => MissingArgumentColon @ span {
        "Argument is missing colon.",
        "Argument should have a colon here." @ span
    },
    E0008 => MissingArgumentValue @ span {
        "Argument is missing value.",
        "Argument should have a value here." @ span
    },
    E0009 => MissingInlineFragmentSelectionSet @ span {
        "Inline fragment is missing selection set.",
        "Inline fragment here should have a selection set." @ span
    },
    E0010 => MissingFragmentName @ span {
        "Fragment definition must have a name.",
        "Fragment definition here does not have a name." @ span
    },
    E0011 => MissingFragmentTypeCondition @ span {
        "Fragment definition must have a type condition.",
        "Fragment definition here does not have a type condition." @ span
    },
    E0012 => MissingFragmentSelectionSet @ span {
        "Fragment definition must have a selection set.",
        "Fragment definition here does not have a selection set." @ span
    },
    E0013 => MissingTypeConditionNamedType @ span {
        "Type condition must have a named type.",
        "Type condition here does not have a named type." @ span
    },
    E0014 => MissingListValueClosingBracket @ second {
        "List value must have closing bracket.",
        "This `[` here ..." @ first,
        "... should have a matching `]` here." @ second
    },
    E0015 => MissingObjectValueClosingBrace @ second {
        "Object value must have closing brace.",
        "This `{{` here ..." @ first,
        "... should have a matching `}}` here." @ second
    },
    E0016 => MissingObjectFieldColon @ span {
        "Object field must have a colon.",
        "This object field here is missing a colon." @ span
    },
    E0017 => MissingObjectFieldValue @ span {
        "Object field must have a value.",
        "This object field here is missing a value." @ span
    },
    E0018 => MissingVariableDefinitionsClosingParenthesis @ second {
        "Variable definitions must have a closing parenthesis.",
        "This `(` here ..." @ first,
        "... must have a matching `)` here." @ second
    },
    E0019 => MissingVariableDefinitionColon @ span {
        "Variable definition must have a colon.",
        "This variable definition here is missing a colon." @ span
    },
    E0020 => MissingVariableDefinitionType @ span {
        "Variable definition must have a type.",
        "This variable definition here is missing a type." @ span
    },
    E0021 => MissingDefaultValue @ span {
        "Default value must have a value.",
        "This default value here is missing a value." @ span
    },
    E0022 => MissingListTypeClosingBracket @ second {
        "List type must have a closing bracket.",
        "This `[` here ..." @ first,
        "... must have a matching `]` here." @ second
    },
    E0023 => MissingListTypeWrappedType @ span {
        "List type must wrap another type.",
        "This list type here is missing a wrapped type." @ span
    },
    E0024 => MissingDirectiveName @ span {
        "Directive must have a name.",
        "This directive here is missing a name." @ span
    },
    E0025 => MissingRootOperationTypeDefinitions @ span {
        "Schema definition must define one or more root operation types.",
        "This schema definition doesn't define any root operation types." @ span
    },
    E0026 => MissingRootOperationTypeDefinitionsClosingBrace @ second {
        "Root operation type definitions must have a closing brace.",
        "This `{{` here ..." @ first,
        "... should have a matching `}}` here." @ second
    },
    E0027 => MissingRootOperationTypeDefinitionColon @ span {
        "Root operation type definition must have a colon.",
        "This root operation type definition here is missing a colon." @ span
    },
    E0028 => MissingRootOperationTypeDefinitionNamedType @ span {
        "Root operation type definition must have a named type.",
        "This root operation type definition here is missing a named type." @ span
    },
    E0029 => MissingScalarTypeDefinitionName @ span {
        "Scalar type definition must have a name.",
        "This scalar type definition here is missing a name." @ span
    },
    E0030 => MissingScalarTypeExtensionName @ span {
        "Scalar type extension must have a name.",
        "This scalar type extension here is missing a name." @ span
    },
    E0031 => MissingScalarTypeExtensionDirectives @ span {
        "Scalar type extension must have one or more directives.",
        "This scalar type extension here is missing directives." @ span
    },
    E0032 => MissingObjectTypeDefinitionName @ span {
        "Object type definition must have a name.",
        "This object type definition here is missing a name." @ span
    },
    E0033 => MissingFirstImplementsInterface @ span {
        "Implemented interfaces must not be empty.",
        "This object type here implements an interface, but its name is missing." @ span
    },
    E0034 => MissingSecondImplementsInterface @ span {
        "Implemented interface name is missing.",
        "This object type here implements another interface, but its name is missing." @ span
    },
    E0035 => MissingFieldsDefinitionClosingBrace @ second {
        "Fields definition must have a closing brace.",
        "This `{{` here ..." @ first,
        "... must have a matching `}}` here." @ second
    },
    E0036 => MissingFieldDefinitionColon @ span {
        "Field definition must have a colon.",
        "This field definition here is missing a colon." @ span
    },
    E0037 => MissingFieldDefinitionType @ span {
        "Field definition must have a type.",
        "This field definition here is missing a type." @ span
    },
    E0038 => MissingArgumentsDefinitionClosingParenthesis @ second {
        "Arguments definition must have a closing parenthesis.",
        "This `(` here ..." @ first,
        "... must have a matching `)` here." @ second
    },
    E0039 => MissingInputValueDefinitionColon @ span {
        "Input value definition must have a colon.",
        "This input value definition here is missing a colon." @ span
    },
    E0040 => MissingInputValueDefinitionType @ span {
        "Input value definition must have a type.",
        "This input value definition here is missing a type." @ span
    },
    E0041 => MissingObjectTypeExtensionName @ span {
        "Object type extension must have a name.",
        "This object type extension here is missing a name." @ span
    },
    E0042 => MissingInterfaceTypeDefinitionName @ span {
        "Interface type definition must have a name.",
        "This interface type definition here is missing a name." @ span
    },
    E0043 => MissingInterfaceTypeExtensionName @ span {
        "Interface type extension must have a name.",
        "This interface type extension here is missing a name." @ span
    },
    E0044 => MissingUnionTypeDefinitionName @ span {
        "Union type definition must have a name.",
        "This union type definition here is missing a name." @ span
    },
    E0045 => MissingFirstUnionMemberType @ span {
        "Union type definition must have one or more member types.",
        "This union type definition here is missing a member type." @ span
    },
    E0046 => MissingSecondUnionMemberType @ span {
        "Union member type must have a name.",
        "This union type definition here defines another member type but its name is missing." @ span
    },
    E0047 => MissingUnionTypeExtensionName @ span {
        "Union type extension must have a name.",
        "This union type extension here is missing a name." @ span
    },
    E0048 => MissingEnumTypeDefinitionName @ span {
        "Enum type definition must have a name.",
        "This enum type definition here is missing a name." @ span
    },
    E0049 => MissingEnumValuesClosingBrace @ second {
        "Enum values must have a closing brace.",
        "This `{{` here ..." @ first,
        "... must have a matching `}}` here." @ second
    },
    E0050 => MissingEnumTypeExtensionName @ span {
        "Enum type extension must have a name.",
        "This enum type extension here is missing a name." @ span
    },
    E0051 => MissingInputObjectTypeDefinitionName @ span {
        "Input object type definition must have a name.",
        "This input object type definition here is missing a name." @ span
    },
    E0052 => MissingInputFieldsDefinitionClosingBrace @ second {
        "Input fields definition must have a closing brace.",
        "This `{{` here ..." @ first,
        "... must have a `}}` here." @ second
    },
    E0053 => MissingInputObjectTypeExtensionName @ span {
        "Input object type extension must have a name.",
        "This input object type extension here is missing a name." @ span
    },
    E0054 => MissingDirectiveDefinitionAt @ span {
        "Directive's name must start with an `@`.",
        "This directive's name here does not start with an `@`." @ span
    },
    E0055 => MissingDirectiveDefinitionName @ span {
        "Directive definition must have a name.",
        "This directive definition here is missing a name." @ span
    },
    E0056 => MissingDirectiveDefinitionLocations @ span {
        "Directive definition must have one or more locations.",
        "This directive definition here is missing locations." @ span
    },
    E0057 => MissingFirstDirectiveLocation @ span {
        "Directive definition must have one or more locations.",
        "This directive definition here is missing a location." @ span
    },
    E0058 => MissingSecondDirectiveLocation @ span {
        "Directive location must be defined.",
        "This directive definition here is missing a location." @ span
    },

    E0100 => UnknownNamedType @ span + name {
        "Named type must exist.",
        "Type `{name}` is referenced to here but never defined." @ span
    },
    E0101 => EmptyType @ span + name {
        "Type must define one or more fields.",
        "Type `{name}` is defined here but the schema doesn't define any fields for it." @ span
    },
    E0102 => DuplicateField @ second + field {
        "Field name must be unique.",
        "Field `{field}` is first defined here ..." @ first,
        "... and later defines the same field again here." @ second
    },
    E0103 => DuplicateExtendedField @ second + name, field {
        "Field name must be unique across extended types.",
        "Type `{name}` first defines field `{field}` here, ..." @ first,
        "... gets extended here ..." @ extension,
        "... and later defines the same field again here." @ second
    },
    E0104 => ReservedFieldName @ span + name {
        "Field name must not start with `__`.",
        "Field `{name}` is defined here and starts with `__`, which is reserved." @ span
    },
    E0105 => FieldNotOutputType @ span + name, ty {
        "Field must be output type.",
        "Field `{name}` is defined here as type `{ty}`, which is an input type." @ span
    },
    E0106 => DuplicateArgumentName @ second + name {
        "Argument name must be unique.",
        "Argument `{name}` is first defined here ..." @ first,
        "... and later defined again here." @ second
    },
    E0107 => ReservedInputValueName @ span + name {
        "Input value name must not start with `__`.",
        "Input value `{name}` is defined here and starts with `__`, which is reserved." @ span
    },
    E0108 => InputValueNotInputType @ span + name, ty {
        "Input value must be input type.",
        "Input value `{name}` is defined here as type `{ty}`, which is an output type." @ span
    },
    E0109 => DuplicateImplementsInterface @ second + name, interface {
        "Type must not implement same interface twice.",
        "Type `{name}` first implements interface `{interface}` here ..." @ first,
        "... and later implements the same interface again here." @ second
    },
    E0110 => ImplementsNonInterfaceType @ span + name, interface {
        "Implemented type must be interface.",
        "Type `{name}` implements type `{interface}` here, but `{interface}` is not an interface." @ span
    },
    E0111 => MissingInheritedInterface @ span + name, interface, inherited {
        "Type must also implement inherited interfaces.",
        "Type `{name}` implements interface `{interface}` here, which requires that `{inherited}` is also implemented." @ span
    },
    E0112 => MissingInterfaceField @ span + name, interface, field {
        "Type must implement field from interface.",
        "Type `{name}` implements interface `{interface}` here, which requires that field `{field}` is implemented." @ span
    },
    E0113 => MissingInterfaceFieldArgument @ span + name, interface, field, argument {
        "Type must implement field with argument from interface.",
        "Type `{name}` implements interface `{interface}` here, which requires that field `{field}` has an argument `{argument}`." @ span
    },
    E0114 => InvalidInterfaceFieldArgumentType @ span + name, interface, field, argument, expected, ty {
        "Type must implement field with argument of same type as interface.",
        "Type `{name}` implements interface `{interface}` here, which requires that field `{field}` has an argument `{argument}` of type `{expected}` instead of `{ty}`." @ span
    },
    E0115 => UnexpectedNonNullExtraInterfaceFieldArgument @ span + name, interface, field, argument, ty {
        "Type must not require extra arguments for fields implementing interface.",
        "Type `{name}` implements interface `{interface}` here and defines a field `{field}` with required argument `{argument}` of type `{ty}` that matches a field in the interface." @ span
    },
    E0116 => NonCovariantInterfaceField @ span + name, interface, field, expected, ty {
        "Implemented type and type from interface must be covariant.",
        "Type `{name}` implements interface `{interface}` that defines a field `{field}` of type `{expected}`, but `{field}` is `{ty}` here, which is incompatible." @ span
    },
    E0117 => SelfReferentialInterface @ span + name {
        "Interface must not implement itself.",
        "Interface `{name}` attempts to implement itself here." @ span
    },
    E0118 => MissingUnionMembers @ span + name {
        "Union must have at least one member type.",
        "Union `{name}` here doesn't have any member types." @ span
    },
    E0119 => DuplicateUnionMember @ second + name {
        "Union members must be unique.",
        "Union member type `{name}` first occurs here ..." @ first,
        "... and later again here." @ second
    },
    E0120 => NonObjectUnionMember @ span + name {
        "Union members must be object types.",
        "Union member type `{name}` is not an object type." @ span
    },
    E0121 => MissingEnumValues @ span + name {
        "Enum must have at least one value.",
        "Enum `{name}` here doesn't have any values." @ span
    },
    E0122 => DuplicateEnumValue @ second + name {
        "Enum values must be unique.",
        "Enum value `{name}` first occurs here ..." @ first,
        "... and later again here." @ second
    },
    E0123 => SelfReferentialInputType @ span + name, field, ty {
        "Input type must not be self-referential.",
        "Input type `{name}` references itself here through field `{field}` of type `{ty}`." @ span
    },
    E0124 => ReservedDirectiveName @ span + name {
        "Directive name must not start with `__`.",
        "Directive `{name}` is defined here, but its name start with `__` which is reserved." @ span
    },
    E0125 => SelfReferentialDirective @ span + name, directive {
        "Directive must not be self-referential.",
        "Directive `{name}` references itself here through directive `{directive}`." @ span
    },
    E0126 => DuplicateTypeName @ second + name {
        "Type name must be unique.",
        "Type `{name}` is first defined here ..." @ first,
        "... and later again here." @ second
    },
    E0126 => DuplicateDirectiveName @ second + name {
        "Directive name must be unique.",
        "Directive `{name}` is first defined here ..." @ first,
        "... and later again here." @ second
    },
    E0127 => DifferentExtensionType @ second + name, first_type, second_type {
        "Type extension must extend type of same kind.",
        "Type `{name}` is first defined as `{first_type}` here ..." @ first,
        "... and later again as `{second_type}` here." @ second
    },
    E0200 => ExpectedNonNullValue @ span + ty {
        "Expected a non-null value.",
        "This should be a `{ty}` here." @ span
    },
    E0201 => ExpectedListValue @ span + ty {
        "Expected a list value.",
        "This should be a `{ty}` here." @ span
    },
    E0202 => ExpectedIntValue @ span + ty {
        "Expected an int value.",
        "This should be a `{ty}` here." @ span
    },
    E0203 => ExpectedFloatValue @ span + ty {
        "Expected a float value.",
        "This should be a `{ty}` here." @ span
    },
    E0204 => ExpectedStringValue @ span + ty {
        "Expected a string value.",
        "This should be a `{ty}` here." @ span
    },
    E0205 => ExpectedBooleanValue @ span + ty {
        "Expected a boolean value.",
        "This should be a `{ty}` here." @ span
    },
    E0206 => ExpectedInputObjectValue @ span + ty {
        "Expected an input object value.",
        "This should be a `{ty}` here." @ span
    },
    E0207 => MissingInputField @ span + name, ty {
        "Expected a value for required input field.",
        "This input object here must have a field `{name}` of type `{ty}`." @ span
    },
    E0208 => UnrecognizedInputField @ span + name, ty {
        "All fields in input object must exist in schema definition.",
        "Value is provided for field `{name}` here, but type `{ty}` has no such field." @ span
    },
    E0209 => UnrecognizedEnumValue @ span + name, value {
        "Enum values must exist in schema definition.",
        "Enum value `{value}` here is not a valid value for enum `{name}`." @ span
    },
    E0300 => DuplicateOperationName @ second + name {
        "Operation definitions must be unique.",
        "Operation `{name}` is first defined here ..." @ first,
        "... and later defined again here." @ second
    },
    E0301 => LoneAnonymousOperation @ span {
        "Anonymous operation definitions must be alone.",
        "Anonymous operation defined here must not coexist with named operation definitions." @ span
    },
    E0303 => UndefinedField @ span + ty, field {
        "Queried field does not exist.",
        "Type `{ty}` does not have a field named `{field}`." @ span
    }
}

#[cfg(feature = "with-ariadne")]
mod with_ariadne {
    use ariadne::{Label, Report, ReportKind, Span};

    use super::Diagnostic;

    impl<S> Into<Report<S>> for Diagnostic<S>
    where
        S: Copy + Span,
    {
        fn into(self) -> Report<S> {
            let mut builder = Report::build(
                ReportKind::Error,
                self.span().source().to_owned(),
                self.span().start(),
            )
            .with_code(self.code())
            .with_message(self.message());
            builder.add_labels(
                self.labels()
                    .into_iter()
                    .map(|(span, message)| Label::new(span).with_message(message)),
            );
            builder.finish()
        }
    }
}
