use async_recursion::async_recursion;
use indexmap::IndexMap;

use std::collections::{HashMap, HashSet};

use graphql_parser::query::{
    Definition, Document, Field, Mutation, OperationDefinition, Query, Selection, SelectionSet,
};

use super::{Error, Intermediate, Resolver, Value};

pub struct Executor<R>
where
    R: Resolver,
{
    resolver: R,
    root: R::Value,
}

impl<R> Executor<R>
where
    R: Resolver,
{
    /// Returns a new executor that will use the given resolver and root value.
    pub fn new(resolver: R, root: R::Value) -> Executor<R> {
        Executor { resolver, root }
    }

    pub async fn execute_request<'a>(
        &self,
        document: Document<'a, String>,
        operation_name: Option<&str>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error> {
        let operation = self.get_operation(&document, operation_name)?;

        match operation {
            OperationDefinition::Query(query) => {
                self.execute_query(&document, query, variable_values, context)
                    .await
            }
            // OperationDefinition::Mutation(mutation) => {
            //     self.execute_mutation(mutation, variable_values, context)
            //         .await
            // }
            _ => todo!(),
        }
    }

    async fn execute_query<'a>(
        &self,
        document: &Document<'a, String>,
        query: &Query<'a, String>,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error> {
        let result = self
            .execute_selection_set(
                document,
                &query.selection_set,
                (),
                &self.root,
                variable_values,
                context,
            )
            .await;

        let _ = variable_values;

        result
    }

    #[async_recursion(?Send)]
    async fn execute_selection_set<'a>(
        &self,
        document: &Document<'a, String>,
        selection_set: &SelectionSet<'a, String>,
        ty: (),
        value: &R::Value,
        variable_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<IndexMap<String, Value>, R::Error>
    where
        'a: 'async_recursion,
    {
        let mut fields = HashSet::new();
        let grouped_field_set =
            self.collect_fields(document, selection_set, &variable_values, &mut fields);

        let mut result_map = IndexMap::<String, Value>::new();

        for (response_key, fields) in grouped_field_set.into_iter() {
            let field_type = ();

            result_map.insert(
                response_key,
                self.execute_field(
                    document,
                    ty,
                    value,
                    fields,
                    field_type,
                    variable_values,
                    context,
                )
                .await?,
            );
        }

        Ok(result_map)
    }

    // async fn execute_mutation(&'a self, mutation: &Mutation<'a, &'a str>) -> Result<(), R::Error> {
    //     Ok(())
    // }

    async fn execute_field<'a>(
        &self,
        document: &Document<'a, String>,
        object_ty: (),
        object_value: &R::Value,
        fields: Vec<Field<'a, String>>,
        field_ty: (),
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
            .resolve_field_value(
                document,
                object_ty,
                object_value,
                field_name,
                &argument_values,
                context,
            )
            .await?;
        self.complete_value(
            document,
            field_ty,
            fields,
            resolved_value,
            variable_values,
            context,
        )
        .await
    }

    async fn resolve_field_value<'a>(
        &self,
        document: &Document<'a, String>,
        object_ty: (),
        object_value: &R::Value,
        field_name: &str,
        argument_values: &HashMap<String, Value>,
        context: &R::Context,
    ) -> Result<Intermediate<R::Value>, R::Error> {
        match self
            .resolver
            .can_resolve(object_ty, object_value, field_name, context)
        {
            true => {
                self.resolver
                    .resolve(
                        object_ty,
                        object_value,
                        field_name,
                        argument_values,
                        context,
                    )
                    .await
            }
            false => Err(Error::custom(format!(
                "Couldn't retrieve field: {} from {:?}",
                field_name, object_value
            ))),
        }
    }

    #[async_recursion(?Send)]
    async fn complete_value<'a>(
        &self,
        document: &Document<'a, String>,
        field_type: (),
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
                            field_type,
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
            Intermediate::Value(_) => {
                todo!("Didn't expect resolved value for field: {}", field.name)
            }
        };

        self.execute_selection_set(
            document,
            &field.selection_set,
            (),
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

                    let fragment_grouped_field_set = self.collect_fields(
                        document,
                        &fragment.selection_set,
                        variable_values,
                        visited_fragments,
                    );

                    for (key, group) in fragment_grouped_field_set {
                        grouped_fields.entry(key).or_default().extend(group);
                    }
                }
                Selection::InlineFragment(fragment) => {}
            }
        }

        grouped_fields
    }

    fn get_operation<'a, 'b>(
        &self,
        document: &'b Document<'a, String>,
        operation_name: Option<&str>,
    ) -> Result<&'b OperationDefinition<'a, String>, R::Error> {
        let mut operations = document.definitions.iter().flat_map(|def| match def {
            Definition::Operation(op) => match op {
                OperationDefinition::Query(_) | OperationDefinition::Mutation(_) => Some(op),
                _ => None,
            },
            _ => None,
        });

        let names = operations
            .clone()
            .flat_map(|op| match op {
                OperationDefinition::Query(query) => query.name.as_deref(),
                OperationDefinition::Mutation(mutation) => mutation.name.as_deref(),
                _ => None,
            })
            .collect::<Vec<&str>>();

        let operation = match operation_name {
            Some(name) => operations.find(|operation| match operation {
                OperationDefinition::Query(query) => {
                    query.name.as_ref().map(|name| name.as_str()) == Some(name)
                }
                OperationDefinition::Mutation(mutation) => {
                    mutation.name.as_ref().map(|name| name.as_str()) == Some(name)
                }
                _ => false,
            }),
            None => {
                let operation = operations.next();

                if let Some(_) = operations.next() {
                    return Err(Error::unspecified_operation(&names));
                }

                operation
            }
        };

        operation.ok_or(match operation_name {
            Some(name) => Error::unknown_operation(name, &names),
            None => Error::unspecified_operation(&names),
        })
    }
}
