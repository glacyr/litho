use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::{Database, DirectiveLocationKind, DirectiveTarget};

pub struct DirectivesAreInValidLocations<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> DirectivesAreInValidLocations<'a, T>
where
    T: Eq + Hash + ToString,
{
    fn check<N>(&self, node: &'a N, diagnostics: &mut Vec<Diagnostic<Span>>)
    where
        N: DirectiveTarget<T>,
    {
        self.check_location(node, node.valid_location(), diagnostics);
    }

    fn check_location<N>(
        &self,
        node: &'a N,
        expected: DirectiveLocationKind,
        diagnostics: &mut Vec<Diagnostic<Span>>,
    ) where
        N: DirectiveTarget<T>,
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

impl<'a, T> Visit<'a, T> for DirectivesAreInValidLocations<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator)
    }

    fn visit_field(&self, node: &'a Arc<Field<T>>, accumulator: &mut Self::Accumulator) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_fragment_definition(
        &self,
        node: &'a Arc<FragmentDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_inline_fragment(
        &self,
        node: &'a InlineFragment<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_variable_definition(
        &self,
        node: &'a VariableDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_schema_definition(
        &self,
        node: &'a SchemaDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_schema_extension(
        &self,
        node: &'a SchemaExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_scalar_type_definition(
        &self,
        node: &'a ScalarTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_scalar_type_extension(
        &self,
        node: &'a ScalarTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_object_type_definition(
        &self,
        node: &'a ObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_object_type_extension(
        &self,
        node: &'a ObjectTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_field_definition(
        &self,
        node: &'a Arc<FieldDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node.as_ref(), accumulator);
    }

    fn visit_arguments_definition(
        &self,
        node: &'a Arc<ArgumentsDefinition<T>>,
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
        node: &'a InterfaceTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_union_type_definition(
        &self,
        node: &'a UnionTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_enum_type_definition(
        &self,
        node: &'a EnumTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_enum_value_definition(
        &self,
        node: &'a EnumValueDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check(node, accumulator);
    }

    fn visit_input_fields_definition(
        &self,
        node: &'a InputFieldsDefinition<T>,
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
