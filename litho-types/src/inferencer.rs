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
    value_type: Vec<Option<Arc<Type<T>>>>,
}

impl<'a, T> InferenceState<'a, T>
where
    T: Eq + Hash,
{
    pub fn new(database: &'a mut Database<T>) -> InferenceState<'a, T> {
        InferenceState {
            database,
            stack: vec![],
            value_type: vec![],
        }
    }
}

pub struct Inferencer;

impl<'ast, T> Visit<'ast, T> for Inferencer
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

    fn visit_fragment_definition(
        &self,
        node: &'ast Arc<FragmentDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = node
            .type_condition
            .ok()
            .and_then(|ty| ty.named_type.ok())
            .map(|ty| ty.0.as_ref());

        accumulator.stack.push(name.cloned());

        if let Some((name, selection_set)) = name.zip(node.selection_set.ok()) {
            accumulator
                .database
                .inference
                .type_by_selection_set
                .insert(selection_set, &Arc::new(name.to_owned()));
        }
    }

    fn post_visit_fragment_definition(
        &self,
        _node: &'ast Arc<FragmentDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.stack.pop();
    }

    fn visit_inline_fragment(
        &self,
        node: &'ast InlineFragment<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        match node.type_condition.as_ref() {
            Some(ty) => {
                let name = ty.named_type.ok().map(|ty| ty.0.as_ref());
                accumulator.stack.push(name.cloned());

                if let Some((name, selection_set)) = name.zip(node.selection_set.ok()) {
                    accumulator
                        .database
                        .inference
                        .type_by_selection_set
                        .insert(selection_set, &Arc::new(name.to_owned()));
                }
            }
            None => {
                accumulator.stack.push(
                    accumulator
                        .stack
                        .last()
                        .map(Option::as_ref)
                        .flatten()
                        .cloned(),
                );
            }
        }
    }

    fn post_visit_inline_fragment(
        &self,
        _node: &'ast InlineFragment<T>,
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

                for argument in arguments.items.iter() {
                    let definition = match definition
                        .definitions
                        .iter()
                        .find(|def| def.name.as_ref() == argument.name.as_ref())
                    {
                        Some(definition) => definition,
                        None => continue,
                    };

                    accumulator
                        .database
                        .inference
                        .definitions_for_arguments
                        .insert(argument, definition);
                }
            }
        } else {
            accumulator.stack.push(None);
        }
    }

    fn visit_argument(&self, node: &'ast Arc<Argument<T>>, accumulator: &mut Self::Accumulator) {
        let ty = accumulator
            .database
            .inference
            .definitions_for_arguments
            .get(node)
            .and_then(|def| {
                let value = node.value.ok();
                let default_value = def
                    .default_value
                    .as_ref()
                    .and_then(|value| value.value.ok());

                if let Some((value, default_value)) = value.zip(default_value) {
                    accumulator
                        .database
                        .inference
                        .default_value_for_values
                        .insert(value, default_value)
                }

                def.ty.ok()
            });

        accumulator.value_type.push(ty.cloned());
    }

    fn post_visit_argument(
        &self,
        _node: &'ast Arc<Argument<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.value_type.pop();
    }

    fn post_visit_field(&self, _node: &'ast Arc<Field<T>>, accumulator: &mut Self::Accumulator) {
        accumulator.stack.pop();
    }

    fn visit_input_value_definition(
        &self,
        node: &'ast Arc<InputValueDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.value_type.push(node.ty.ok().cloned());
    }

    fn post_visit_input_value_definition(
        &self,
        _node: &'ast Arc<InputValueDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.value_type.pop();
    }

    fn visit_value(&self, node: &'ast Arc<Value<T>>, accumulator: &mut Self::Accumulator) {
        if let Some(ty) = accumulator.value_type.last().and_then(|ty| ty.as_ref()) {
            accumulator
                .database
                .inference
                .types_for_values
                .insert(node, ty);
        }
    }

    fn visit_list_value(&self, _node: &'ast ListValue<T>, accumulator: &mut Self::Accumulator) {
        let ty = accumulator
            .value_type
            .last()
            .and_then(|ty| ty.as_ref())
            .and_then(|ty| ty.list_value_type())
            .map(|ty| ty.clone());

        accumulator.value_type.push(ty);
    }

    fn post_visit_list_value(
        &self,
        _node: &'ast ListValue<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.value_type.pop();
    }

    fn visit_object_field(&self, node: &'ast ObjectField<T>, accumulator: &mut Self::Accumulator) {
        let ty = accumulator
            .value_type
            .last()
            .and_then(|ty| ty.as_ref())
            .and_then(|ty| ty.name())
            .and_then(|ty| {
                accumulator
                    .database
                    .input_value_definitions_by_name(ty, node.name.as_ref())
                    .next()
            })
            .and_then(|field| field.ty.ok().cloned());

        accumulator.value_type.push(ty);
    }

    fn post_visit_object_field(
        &self,
        _node: &'ast ObjectField<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.value_type.pop();
    }

    fn visit_variable_definition(
        &self,
        node: &'ast VariableDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.value_type.push(node.ty.ok().cloned());
    }

    fn post_visit_variable_definition(
        &self,
        _node: &'ast VariableDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.value_type.pop();
    }

    fn visit_fragment_spread(
        &self,
        node: &'ast Arc<FragmentSpread<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let definition = match accumulator
            .database
            .fragments
            .by_name(node.fragment_name.as_ref())
            .next()
        {
            Some(definition) => definition,
            None => return,
        };

        accumulator
            .database
            .usages
            .fragments
            .track(definition, node);
    }

    fn visit_directive(&self, node: &'ast Arc<Directive<T>>, accumulator: &mut Self::Accumulator) {
        let name = match node.name.ok() {
            Some(name) => name,
            None => return,
        };

        let definition = accumulator
            .database
            .directive_definitions_by_name(name.as_ref())
            .next()
            .cloned();

        if let Some(definition) = definition {
            accumulator
                .database
                .inference
                .definition_for_directives
                .insert(node, &definition);
        }
    }
}
