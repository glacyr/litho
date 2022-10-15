use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;

use super::Database;

pub struct State<'a, T>
where
    T: Eq + Hash,
{
    database: &'a mut Database<T>,
    stack: Vec<Option<T>>,
}

impl<'a, T> State<'a, T>
where
    T: Eq + Hash,
{
    pub fn new(database: &'a mut Database<T>) -> State<'a, T> {
        State {
            database,
            stack: vec![],
        }
    }
}

pub struct Inference;

impl<'ast, T> Visit<'ast, T> for Inference
where
    T: From<&'static str> + Clone + std::fmt::Debug + Eq + Hash + 'ast,
{
    type Accumulator = State<'ast, T>;

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
            .type_by_selection_set
            .insert(Arc::as_ptr(selection_set) as usize, name.to_owned());

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
                    .field_definitions_by_field
                    .insert(Arc::as_ptr(node) as usize, definition.clone());
            }

            let ty = definition
                .as_ref()
                .and_then(|def| def.ty.ok())
                .and_then(|ty| ty.name())
                .cloned();

            if let Some((set, ty)) = node.selection_set.as_ref().zip(ty.as_ref()) {
                accumulator
                    .database
                    .type_by_selection_set
                    .insert(Arc::as_ptr(set) as usize, ty.to_owned());
            }

            accumulator.stack.push(ty);

            if let Some((arguments, definition)) = node
                .arguments
                .as_ref()
                .zip(definition.and_then(|def| def.arguments_definition.to_owned()))
            {
                accumulator
                    .database
                    .definition_for_arguments
                    .insert(Arc::as_ptr(arguments) as usize, definition);
            }
        } else {
            accumulator.stack.push(None);
        }
    }

    fn post_visit_field(&self, _node: &'ast Arc<Field<T>>, accumulator: &mut Self::Accumulator) {
        accumulator.stack.pop();
    }
}
