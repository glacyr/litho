use async_recursion::async_recursion;
use indexmap::IndexMap;

use std::collections::{HashMap, HashSet};

use graphql_parser::query::{
    Definition, Document, Field, Mutation, OperationDefinition, Query, Selection, SelectionSet,
    Subscription, TypeCondition,
};

use super::{Error, Intermediate, Resolver, Root, Typename, Value};

/// Executor of a resolver that implements the GraphQL specification for
/// execution.
pub struct Executor<R>
where
    R: Resolver,
{
    resolver: R,
}

impl<R> Executor<R>
where
    R: Resolver,
{
    /// Returns a new executor that will use the given resolver and root value.
    pub fn new(resolver: R) -> Executor<R> {
        Executor { resolver }
    }

    /// Executes a request with the given document, operation, variable values
    /// and context and returns an ordered map with the resulting value. The
    /// context is passed along to the resolver in [`Resolver::can_resolve`] and
    /// [`Resolver::resolve`]. Depending on the type of operation (`query`,
    /// `mutation`, `subscription`), this method will use the [`Root`] trait to
    /// get a root value that's passed to the first level of resolvers .
    pub async fn execute_request<'a>(
        &self,
        document: Document<'a, String>,
        operation_name: Option<&str>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error> {
        let operation = self.get_operation(&document, operation_name)?;

        match operation {
            OperationDefinition::SelectionSet(selection_set) => {
                self.execute_selection_set(
                    &document,
                    selection_set,
                    &R::Value::query(),
                    variable_values,
                    context,
                )
                .await
            }
            OperationDefinition::Query(query) => {
                self.execute_query(&document, query, variable_values, context)
                    .await
            }
            OperationDefinition::Mutation(mutation) => {
                self.execute_mutation(&document, mutation, variable_values, context)
                    .await
            }
            OperationDefinition::Subscription(subscription) => {
                self.execute_subscription(&document, subscription, variable_values, context)
                    .await
            }
        }
    }

    async fn execute_query<'a>(
        &self,
        document: &Document<'a, String>,
        query: &Query<'a, String>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error> {
        self.execute_selection_set(
            document,
            &query.selection_set,
            &R::Value::query(),
            variable_values,
            context,
        )
        .await
    }

    async fn execute_mutation<'a>(
        &self,
        document: &Document<'a, String>,
        mutation: &Mutation<'a, String>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error> {
        self.execute_selection_set(
            document,
            &mutation.selection_set,
            &R::Value::query(),
            variable_values,
            context,
        )
        .await
    }

    async fn execute_subscription<'a>(
        &self,
        document: &Document<'a, String>,
        subscription: &Subscription<'a, String>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error> {
        self.execute_selection_set(
            document,
            &subscription.selection_set,
            &R::Value::subscription(),
            variable_values,
            context,
        )
        .await
    }

    #[async_recursion(?Send)]
    async fn execute_selection_set<'a>(
        &self,
        document: &Document<'a, String>,
        selection_set: &SelectionSet<'a, String>,
        value: &R::Value,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error>
    where
        'a: 'async_recursion,
    {
        let mut fields = HashSet::new();
        let grouped_field_set = self.collect_fields(
            document,
            value,
            selection_set,
            &variable_values,
            &mut fields,
        );

        let mut result_map = IndexMap::<String, Value>::new();

        for (response_key, fields) in grouped_field_set.into_iter() {
            result_map.insert(
                response_key,
                self.execute_field(document, value, fields, variable_values, context)
                    .await?,
            );
        }

        Ok(result_map)
    }

    async fn execute_field<'a>(
        &self,
        document: &Document<'a, String>,
        object_value: &R::Value,
        fields: Vec<Field<'a, String>>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<Value, R::Error> {
        // ...
        let field = fields.first().unwrap();
        let field_name = &field.name;
        let argument_values = HashMap::new();
        // self.coerce_argument_values(object_ty, field, variable_values);
        // let argument_values = ();
        let resolved_value = self
            .resolve_field_value(object_value, field_name, &argument_values, context)
            .await?;
        self.complete_value(document, fields, resolved_value, variable_values, context)
            .await
    }

    async fn resolve_field_value<'a>(
        &self,
        object_value: &R::Value,
        field_name: &str,
        argument_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<Intermediate<R::Value>, R::Error> {
        if field_name == "__typename" {
            return Ok(Intermediate::Value(object_value.typename().into()));
        }

        match self.resolver.can_resolve(object_value, field_name, context) {
            true => {
                self.resolver
                    .resolve(object_value, field_name, argument_values, context)
                    .await
            }
            false => Err(Error::unknown_field(&object_value.typename(), field_name)),
        }
    }

    #[async_recursion(?Send)]
    async fn complete_value<'a>(
        &self,
        document: &Document<'a, String>,
        fields: Vec<Field<'a, String>>,
        result: Intermediate<R::Value>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<Value, R::Error>
    where
        'a: 'async_recursion,
    {
        let orig_fields = fields.clone();
        let field = fields.into_iter().next().unwrap();

        if field.selection_set.items.is_empty() {
            return match result {
                Intermediate::Collection(_) | Intermediate::Object(_) => todo!(),
                Intermediate::Value(value) => Ok(value),
            };
        }

        let result = match result {
            Intermediate::Collection(collection) => {
                let mut results = vec![];

                for value in collection {
                    results.push(
                        self.complete_value(
                            document,
                            orig_fields.clone(),
                            value,
                            variable_values,
                            context,
                        )
                        .await?,
                    );
                }

                return Ok(results.into());
            }
            Intermediate::Object(object) => object,
            Intermediate::Value(Value::Null) => return Ok(Value::Null),
            Intermediate::Value(_) => {
                todo!("Didn't expect resolved value for field: {}", field.name)
            }
        };

        self.execute_selection_set(
            document,
            &field.selection_set,
            &result,
            variable_values,
            context,
        )
        .await
        .map(|result| result.into_iter().collect())
    }

    fn collect_fields<'a>(
        &self,
        document: &Document<'a, String>,
        value: &R::Value,
        selection_set: &SelectionSet<'a, String>,
        variable_values: &HashMap<String, Value>,
        visited_fragments: &mut HashSet<String>,
    ) -> IndexMap<String, Vec<Field<'a, String>>> {
        let mut grouped_fields = IndexMap::<String, Vec<Field<'a, String>>>::new();

        for selection in selection_set.items.iter() {
            match selection {
                Selection::Field(field) => {
                    let response_key = field.alias.as_ref().unwrap_or(&field.name).to_owned();
                    grouped_fields
                        .entry(response_key)
                        .or_default()
                        .push(field.to_owned());
                }
                Selection::FragmentSpread(spread) => {
                    if visited_fragments.contains(&spread.fragment_name) {
                        continue;
                    }

                    visited_fragments.insert(spread.fragment_name.to_owned());

                    let fragment = document
                        .definitions
                        .iter()
                        .find_map(|definition| match definition {
                            Definition::Fragment(fragment)
                                if fragment.name == spread.fragment_name =>
                            {
                                Some(fragment)
                            }
                            _ => None,
                        })
                        .unwrap();

                    if !self.does_fragment_apply(value, &fragment.type_condition) {
                        continue;
                    }

                    let fragment_grouped_field_set = self.collect_fields(
                        document,
                        value,
                        &fragment.selection_set,
                        variable_values,
                        visited_fragments,
                    );

                    for (key, group) in fragment_grouped_field_set {
                        grouped_fields.entry(key).or_default().extend(group);
                    }
                }
                Selection::InlineFragment(fragment) => {
                    // Let `fragmentType` be the type condition on `selection`.
                    let fragment_type = fragment.type_condition.as_ref();

                    // If `fragmentType` is not `null` and
                    // `DoesFragmentTypeApply(objectType, fragmentType)` is
                    // `false`, continue with the next `selection` in
                    // `selectionSet`.
                    match fragment_type {
                        Some(fragment_type) if self.does_fragment_apply(value, fragment_type) => {
                            continue
                        }
                        _ => {}
                    }

                    // Let `fragmentSelectionSet` be the top-level selection
                    // set of `selection`.
                    let fragment_selection_set = &fragment.selection_set;

                    // Let `fragmentGroupedFieldSet` be the result of calling
                    // `CollectFields(objectType, fragmentSelectionSet,
                    // variableValues, visitedFragments)`.
                    let fragment_grouped_field_set = self.collect_fields(
                        document,
                        value,
                        fragment_selection_set,
                        variable_values,
                        visited_fragments,
                    );

                    // For each `fragmentGroup` in `fragmentGroupedFieldSet`:
                    // 1. Let `responseKey` be the respone key shared by all
                    //    fields in `fragmentGroup`.
                    for (response_key, fragment_group) in fragment_grouped_field_set {
                        // 2. Let `groupForResponseKey` be the list in
                        //    `groupedFields` for `responseKey`; if no such list
                        //    exists, create it as an empty list.
                        let group_for_response_key =
                            grouped_fields.entry(response_key).or_default();

                        // 3. Append all items in `fragmentGroup` to
                        //    `groupForResponseKey`.
                        group_for_response_key.extend(fragment_group);
                    }
                }
            }
        }

        grouped_fields
    }

    fn does_fragment_apply(&self, value: &R::Value, fragment_type: &TypeCondition<String>) -> bool {
        match fragment_type {
            TypeCondition::On(name) => name == &value.typename(),
        }
    }

    fn get_operation<'a, 'b>(
        &self,
        document: &'b Document<'a, String>,
        operation_name: Option<&str>,
    ) -> Result<&'b OperationDefinition<'a, String>, R::Error> {
        let names = document
            .definitions
            .iter()
            .flat_map(|definition| match definition {
                Definition::Operation(op) => match op {
                    OperationDefinition::Query(query) => query.name.as_deref(),
                    OperationDefinition::Mutation(mutation) => mutation.name.as_deref(),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<&str>>();

        let mut applicable = document
            .definitions
            .iter()
            .flat_map(|definition| match definition {
                Definition::Operation(op) => match op {
                    OperationDefinition::Query(_)
                    | OperationDefinition::Mutation(_)
                    | OperationDefinition::SelectionSet(_) => Some(op),
                    _ => None,
                },
                _ => None,
            });

        match operation_name {
            Some(name) => applicable
                .find(|operation| match operation {
                    OperationDefinition::Query(query) => {
                        query.name.as_ref().map(|name| name.as_str()) == Some(name)
                    }
                    OperationDefinition::Mutation(mutation) => {
                        mutation.name.as_ref().map(|name| name.as_str()) == Some(name)
                    }
                    OperationDefinition::Subscription(subscription) => {
                        subscription.name.as_ref().map(|name| name.as_str()) == Some(name)
                    }
                    _ => false,
                })
                .ok_or(Error::unknown_operation(name, &names)),
            None => {
                let operation = applicable.next();

                if let Some(_) = applicable.next() {
                    return Err(Error::unspecified_operation(&names));
                }

                operation.ok_or(Error::missing_operation())
            }
        }
    }
}
