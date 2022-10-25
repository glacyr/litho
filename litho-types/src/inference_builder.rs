use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;

use super::Database;

pub struct InferenceState<'a, T>
where
    T: Eq + Hash,
{
    database: &'a mut Database<T>,
    stack: Vec<Option<T>>,
}

impl<'a, T> InferenceState<'a, T>
where
    T: Eq + Hash,
{
    pub fn new(database: &'a mut Database<T>) -> InferenceState<'a, T> {
        InferenceState {
            database,
            stack: vec![],
        }
    }
}

pub struct InferenceBuilder;

impl<'ast, T> Visit<'ast, T> for InferenceBuilder
where
    T: From<&'static str> + Clone + Eq + Hash + 'ast,
{
    type Accumulator = InferenceState<'ast, T>;

    fn visit_operation_definition(
        &self,
        node: &'ast Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let selection_set = match node.selection_set.ok() {
            Some(set) => set,
            None => {
                accumulator.stack.push(None);

                return;
            }
        };

        let name = T::from(match node.ty {
            Some(OperationType::Query(_)) | None => "Query",
            Some(OperationType::Mutation(_)) => "Mutation",
            Some(OperationType::Subscription(_)) => "Subscription",
        });

        accumulator
            .database
            .inference
            .type_by_selection_set
            .insert(selection_set, &Arc::new(name.to_owned()));

        accumulator.stack.push(Some(name));
    }

    fn post_visit_operation_definition(
        &self,
        _node: &'ast Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.stack.pop();
    }

    fn visit_field(&self, node: &'ast Arc<Field<T>>, accumulator: &mut Self::Accumulator) {
        let ty = accumulator.stack.last().into_iter().flatten().next();
        let name = node.name.ok();

        if let Some((ty, name)) = ty.zip(name) {
            let definition = accumulator
                .database
                .field_definitions_by_name(ty, name.as_ref())
                .next()
                .cloned();

            if let Some(definition) = definition.as_ref() {
                accumulator
                    .database
                    .inference
                    .field_definitions_by_field
                    .insert(node, definition);
            }

            let ty = definition
                .as_ref()
                .and_then(|def| def.ty.ok())
                .and_then(|ty| ty.name())
                .cloned();

            if let Some((set, ty)) = node.selection_set.as_ref().zip(ty.as_ref()) {
                accumulator
                    .database
                    .inference
                    .type_by_selection_set
                    .insert(set, &Arc::new(ty.to_owned()));
            }

            accumulator.stack.push(ty);

            if let Some((arguments, definition)) = node.arguments.as_ref().zip(
                definition
                    .as_ref()
                    .and_then(|def| def.arguments_definition.as_ref()),
            ) {
                accumulator
                    .database
                    .inference
                    .definition_for_arguments
                    .insert(arguments, definition);
            }
        } else {
            accumulator.stack.push(None);
        }
    }

    fn post_visit_field(&self, _node: &'ast Arc<Field<T>>, accumulator: &mut Self::Accumulator) {
        accumulator.stack.pop();
    }

    fn visit_input_value_definition(
        &self,
        node: &'ast Arc<InputValueDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.ty.ok().zip(
            node.default_value
                .as_ref()
                .and_then(|value| value.value.ok()),
        ) {
            Some((ty, value)) => {
                accumulator
                    .database
                    .inference
                    .types_for_values
                    .insert(value, ty);
            }
            None => {}
        }
    }
}
