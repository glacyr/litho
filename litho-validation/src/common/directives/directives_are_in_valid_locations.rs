use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::{Database, DirectiveLocationKind, DirectiveTarget};

pub struct DirectivesAreInValidLocations<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> DirectivesAreInValidLocations<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    fn check<N>(&self, node: &'ast N, diagnostics: &mut Vec<Diagnostic<Span>>)
    where
        N: DirectiveTarget<'a, T>,
    {
        self.check_location(node, node.valid_location(), diagnostics);
    }

    fn check_location<N>(
        &self,
        node: &'ast N,
        expected: DirectiveLocationKind,
        diagnostics: &mut Vec<Diagnostic<Span>>,
    ) where
        N: DirectiveTarget<'a, T>,
    {
        for directive in node
            .directives()
            .into_iter()
            .flat_map(|directives| directives.directives.iter())
        {
            let Some(name) = directive.name.ok() else {
                return;
            };

            let Some(definition) = self.0.inference.definition_for_directive(directive) else {
                return;
            };

            let locations = definition
                .locations
                .ok()
                .into_iter()
                .flat_map(|locations| locations.locations())
                .map(Into::into)
                .collect::<Vec<DirectiveLocationKind>>();

            if !locations.contains(&expected) {
                diagnostics.push(Diagnostic::directive_in_invalid_location(
                    name.as_ref().to_string(),
                    expected.to_string(),
                    locations
                        .into_iter()
                        .map(|location| location.to_string())
                        .collect::<Vec<_>>()
                        .join(" | "),
                    directive.name.span(),
                ));
            }
        }
    }
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for DirectivesAreInValidLocations<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'ast Shared<'a, T, OperationDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator)
    }

    fn visit_field(
        &self,
        node: &'ast Shared<'a, T, Field<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_fragment_definition(
        &self,
        node: &'ast Shared<'a, T, FragmentDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_fragment_spread(
        &self,
        node: &'ast Shared<'a, T, FragmentSpread<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_inline_fragment(
        &self,
        node: &'ast InlineFragment<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_variable_definition(
        &self,
        node: &'ast VariableDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_schema_definition(
        &self,
        node: &'ast SchemaDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_schema_extension(
        &self,
        node: &'ast SchemaExtension<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_scalar_type_definition(
        &self,
        node: &'ast ScalarTypeDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_scalar_type_extension(
        &self,
        node: &'ast ScalarTypeExtension<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_object_type_definition(
        &self,
        node: &'ast ObjectTypeDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_object_type_extension(
        &self,
        node: &'ast ObjectTypeExtension<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_field_definition(
        &self,
        node: &'ast Shared<'a, T, FieldDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_arguments_definition(
        &self,
        node: &'ast Shared<'a, T, ArgumentsDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        for node in node.definitions.iter() {
            self.check_location(
                node.as_ref(),
                DirectiveLocationKind::ArgumentDefinition,
                accumulator,
            );
        }
    }

    fn visit_interface_type_definition(
        &self,
        node: &'ast InterfaceTypeDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_union_type_definition(
        &self,
        node: &'ast UnionTypeDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_enum_type_definition(
        &self,
        node: &'ast EnumTypeDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_enum_value_definition(
        &self,
        node: &'ast EnumValueDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_input_fields_definition(
        &self,
        node: &'ast InputFieldsDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        for node in node.definitions.iter() {
            self.check_location(
                node.as_ref(),
                DirectiveLocationKind::InputFieldDefinition,
                accumulator,
            );
        }
    }
}
