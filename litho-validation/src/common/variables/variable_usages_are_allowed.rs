use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::once;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct VariableUsagesAreAllowed<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> Visit<'a, T> for VariableUsagesAreAllowed<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(variable_definitions) = node.variable_definitions.as_ref() else {
            return
        };

        let variable_definitions = variable_definitions
            .variable_definitions
            .iter()
            .map(|def| (def.variable.name.as_ref(), def.as_ref()))
            .collect();

        node.traverse(
            &VariableUsagesAreAllowedInOperation {
                database: self.0,
                variable_definitions: &variable_definitions,
            },
            accumulator,
        )
    }
}

fn is_variable_usage_allowed<T>(
    database: &Database<T>,
    definition: &VariableDefinition<T>,
    value: &Arc<Value<T>>,
) -> bool
where
    T: Eq + Hash,
{
    let definition_default_value = definition
        .default_value
        .as_ref()
        .and_then(|value| value.value.ok());

    let location_default_value = database.inference.default_value_for_values.get(value);

    let Some(actual) = definition.ty.ok() else {
        return true
    };

    let Some(expected) = database.inference.types_for_values.get(value) else {
        return true
    };

    let expected_nullable = match (definition_default_value, location_default_value) {
        (None, None) => expected,
        (_, _) => expected.as_nullable(),
    };

    are_types_compatible(actual, expected_nullable)
}

fn are_types_compatible<T>(variable_type: &Type<T>, location_type: &Type<T>) -> bool
where
    T: Eq,
{
    match (location_type, variable_type) {
        (Type::NonNull(location_type), Type::NonNull(variable_type)) => {
            are_types_compatible(variable_type.ty.as_ref(), location_type.ty.as_ref())
        }
        (Type::NonNull(_), _) => false,
        (location_type, Type::NonNull(variable_type)) => {
            are_types_compatible(variable_type.ty.as_ref(), location_type)
        }
        (Type::List(location_type), Type::List(variable_type)) => {
            match location_type.ty.ok().zip(variable_type.ty.ok()) {
                Some((location_type, variable_type)) => {
                    are_types_compatible(variable_type, location_type)
                }
                None => true,
            }
        }
        (_, Type::List(_)) => false,
        (lhs, rhs) => lhs.is_invariant(rhs),
    }
}

pub struct VariableUsagesAreAllowedInOperation<'a, T>
where
    T: Eq + Hash,
{
    database: &'a Database<T>,
    variable_definitions: &'a HashMap<&'a T, &'a VariableDefinition<T>>,
}

impl<'a, T> Visit<'a, T> for VariableUsagesAreAllowedInOperation<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_value(&self, node: &'a Arc<Value<T>>, accumulator: &mut Self::Accumulator) {
        let variable = match node.as_ref() {
            Value::Variable(variable) => variable,
            _ => return,
        };

        let Some(definition) = self.variable_definitions.get(variable.name.as_ref()) else {
            return
        };

        let Some(actual) = definition.ty.ok() else {
            return
        };

        let Some(expected) = self.database.inference.types_for_values.get(node) else {
            return
        };

        if !is_variable_usage_allowed(self.database, definition, node) {
            accumulator.push(Diagnostic::incompatible_variable(
                variable.name.as_ref().to_string(),
                actual.to_string(),
                expected.to_string(),
                definition.ty.span(),
                node.span(),
            ))
        }
    }

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(definition) = self
            .database
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next() else {
            return
        };

        definition.traverse(
            &VariableUsagesAreAllowedInFragment {
                database: self.database,
                variable_definitions: self.variable_definitions,
                fragment_name: node.fragment_name.as_ref(),
                fragment_span: node.fragment_name.span(),
                stack: vec![node.fragment_name.as_ref()].into_iter().collect(),
            },
            accumulator,
        )
    }
}

pub struct VariableUsagesAreAllowedInFragment<'a, T>
where
    T: Eq + Hash,
{
    database: &'a Database<T>,
    variable_definitions: &'a HashMap<&'a T, &'a VariableDefinition<T>>,
    fragment_name: &'a T,
    fragment_span: Span,
    stack: HashSet<&'a T>,
}

impl<'a, T> Visit<'a, T> for VariableUsagesAreAllowedInFragment<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_value(&self, node: &'a Arc<Value<T>>, accumulator: &mut Self::Accumulator) {
        let variable = match node.as_ref() {
            Value::Variable(variable) => variable,
            _ => return,
        };

        let Some(definition) = self.variable_definitions.get(variable.name.as_ref()) else {
            return
        };

        let Some(actual) = definition.ty.ok() else {
            return
        };

        let Some(expected) = self.database.inference.types_for_values.get(node) else {
            return
        };

        if !is_variable_usage_allowed(self.database, definition, node) {
            accumulator.push(Diagnostic::incompatible_variable_in_fragment(
                self.fragment_name.to_string(),
                variable.name.as_ref().to_string(),
                actual.to_string(),
                expected.to_string(),
                self.fragment_span,
                definition.ty.span(),
                node.span(),
            ))
        }
    }

    fn visit_fragment_spread(
        &self,
        node: &'a Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        if self.stack.contains(node.fragment_name.as_ref()) {
            return;
        }

        let Some(definition) = self
            .database
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next() else {
            return
        };

        definition.traverse(
            &VariableUsagesAreAllowedInFragment {
                stack: self
                    .stack
                    .iter()
                    .copied()
                    .chain(once(node.fragment_name.as_ref()))
                    .collect(),
                ..*self
            },
            accumulator,
        )
    }
}
