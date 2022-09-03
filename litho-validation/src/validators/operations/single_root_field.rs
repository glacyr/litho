use graphql_parser::query::{
    Definition, Field, FragmentSpread, InlineFragment, OperationDefinition, Text, TypeCondition,
};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.2.3.1 Single Root Field
///
/// ## Formal Specification
/// - For each subscription operation definition `subscription` in the document
/// - Let `subscriptionType` be the root Subscription type in `schema`.
/// - Let `selectionSet` be the top level selection set on `subscription`.
/// - Let `variableValues` be the empty set.
/// - Let `groupedFieldSet` be the result of
///   `CollectFields(subscriptionType, selectionSet, variableValues)`.
/// - `groupedFieldSet` must have exactly one entry, which must not be an
///   introspection field.
pub struct SingleRootField;

struct CollectFields<'v, 'a, T>
where
    T: Text<'a>,
{
    document: &'v query::Document<'a, T>,
    ty: &'v str,
}

impl<'v, 'a, T> CollectFields<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    pub fn new(document: &'v query::Document<'a, T>, ty: &'v str) -> CollectFields<'v, 'a, T> {
        CollectFields { document, ty }
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for CollectFields<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<&'v Field<'a, T>>;

    fn visit_field(
        &self,
        field: &'v Field<'a, T>,
        _schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.push(field);
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for CollectFields<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    fn traverse_inline_fragment(
        &self,
        inline_fragment: &'v InlineFragment<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        match &inline_fragment.type_condition {
            Some(TypeCondition::On(ty))
                if schema
                    .possible_types(ty.as_ref())
                    .find(|&ty| ty == self.ty)
                    .is_some() =>
            {
                let scope = scope.inline_fragment(ty.as_ref(), inline_fragment.span());
                self.traverse_selection_set(
                    &inline_fragment.selection_set,
                    schema,
                    &scope,
                    accumulator,
                );
            }
            _ => {}
        }
    }

    fn traverse_fragment_spread(
        &self,
        fragment_spread: &'v FragmentSpread<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let definition = match self
            .document
            .fragment_definition(fragment_spread.fragment_name.as_ref())
        {
            Some(definition) => definition,
            None => return,
        };

        let ty = match &definition.type_condition {
            TypeCondition::On(ty) => ty,
        };

        if schema
            .possible_types(ty.as_ref())
            .find(|&ty| ty == self.ty)
            .is_some()
        {
            self.traverse_fragment_definition(definition, schema, scope, accumulator);
        }
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for SingleRootField
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_document(
        &self,
        document: &'v query::Document<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        for definition in document.definitions.iter() {
            match definition {
                Definition::Operation(OperationDefinition::Subscription(subscription)) => {
                    let mut fields = vec![];
                    CollectFields::new(document, "Subscription").traverse_subscription(
                        subscription,
                        schema,
                        scope,
                        &mut fields,
                    );

                    if let [first, second, ..] = &fields[..] {
                        accumulator.push(Error::MultipleSubscriptionRoots {
                            first_name: first.name.as_ref(),
                            first_span: first.span(),
                            second_name: second.name.as_ref(),
                            second_span: second.span(),
                        })
                    }
                }
                _ => {}
            }
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for SingleRootField
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_single_root_field() {
        crate::tests::assert_err(
            r#"
        interface Bar {
            bar: Boolean!
        }

        type Subscription implements Bar {
            foo: Int!
            bar: Boolean!
        }
        "#,
            r#"
        fragment example on Bar {
            bar
        }

        subscription {
            ... example
            ... on Subscription {
                foo
            }
        }
        "#,
            r#"
        Error: 5.2.3.1 Single Root Field

          × Subscriptions should have exactly one field.
            ╭────
          1 │ fragment example on Bar {
          2 │     bar
            ·     ─┬─
            ·      ╰── Subscriptions should have exactly one field but first field `bar` is referenced here ...
            ·
          3 │ }
          4 │ 
          5 │ subscription {
          6 │     ... example
          7 │     ... on Subscription {
          8 │         foo
            ·         ─┬─
            ·          ╰── ... and second field `foo` is referenced here.
            ·
          9 │     }
         10 │ }
            ╰────
        "#,
        )
    }
}
