use graphql_parser::query::{Directive, Mutation, Query, Subscription, Text};
use graphql_parser::schema::{DirectiveLocation, Document};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.7.2 Directives Are In Valid Locations
/// ## Formal Specification
/// - For every `directive` in a document.
/// - Let `directiveName` be the name of `directive`.
/// - Let `directiveDefinition` be the directive named `directiveName`.
/// - Let `locations` be the valid locations for `directiveDefinition`.
/// - Let `adjacent` be the AST node the directive affects.
/// - `adjacent` must be represented by an item within `locations`.
pub struct DirectivesAreInvalidLocations;

impl DirectivesAreInvalidLocations {
    pub fn check_directives<'v, 'a, T>(
        &self,
        directives: &'v Vec<Directive<'a, T>>,
        schema: &'v Document<'a, T>,
        location: DirectiveLocation,
        errors: &mut Vec<Error<'v, 'a, T>>,
    ) where
        'a: 'v,
        T: Text<'a>,
    {
        for directive in directives.iter() {
            let definition = match schema.directive_definition(&directive.name) {
                Some(definition) => definition,
                None => continue,
            };

            if !definition.locations.contains(&location) {
                errors.push(Error::InvalidDirectiveLocation {
                    name: directive.name.as_ref(),
                    span: directive.span(),
                });
            }
        }
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for DirectivesAreInvalidLocations
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_query(
        &self,
        query: &'v Query<'a, T>,
        schema: &'v Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check_directives(
            &query.directives,
            schema,
            DirectiveLocation::Query,
            accumulator,
        )
    }

    fn visit_mutation(
        &self,
        mutation: &'v Mutation<'a, T>,
        schema: &'v Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check_directives(
            &mutation.directives,
            schema,
            DirectiveLocation::Mutation,
            accumulator,
        )
    }

    fn visit_subscription(
        &self,
        subscription: &'v Subscription<'a, T>,
        schema: &'v Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check_directives(
            &subscription.directives,
            schema,
            DirectiveLocation::Subscription,
            accumulator,
        )
    }

    fn visit_field(
        &self,
        field: &'v graphql_parser::query::Field<'a, T>,
        schema: &'v Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check_directives(
            &field.directives,
            schema,
            DirectiveLocation::Field,
            accumulator,
        )
    }

    fn visit_fragment_definition(
        &self,
        fragment_definition: &'v graphql_parser::query::FragmentDefinition<'a, T>,
        schema: &'v Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check_directives(
            &fragment_definition.directives,
            schema,
            DirectiveLocation::FragmentDefinition,
            accumulator,
        )
    }

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v graphql_parser::query::FragmentSpread<'a, T>,
        schema: &'v Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check_directives(
            &fragment_spread.directives,
            schema,
            DirectiveLocation::FragmentSpread,
            accumulator,
        )
    }

    fn visit_inline_fragment(
        &self,
        inline_fragment: &'v graphql_parser::query::InlineFragment<'a, T>,
        schema: &'v Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.check_directives(
            &inline_fragment.directives,
            schema,
            DirectiveLocation::InlineFragment,
            accumulator,
        )
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for DirectivesAreInvalidLocations
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_directives_are_in_valid_locations_query() {
        crate::tests::assert_ok(
            r#"
        directive @example on QUERY

        type Query {
            foobar: String!
        }
        "#,
            r#"
        query @example {
            foobar
        }
        "#,
        );

        crate::tests::assert_err(
            r#"
        directive @example on FIELD

        type Query {
            foobar: String!
        }
        "#,
            r#"
        query @example {
            foobar
        }
        "#,
            r#"
        Error: 5.7.2 Directives Are In Valid Locations

          × Directive `example` cannot be used in this location.
           ╭────
         1 │ query @example {
           ·       ───┬────
           ·          ╰── Directive `example` is used here but cannot be used in this location.
           ·
         2 │     foobar
         3 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_directives_are_in_valid_locations_mutation() {
        crate::tests::assert_ok(
            r#"
        directive @example on MUTATION

        type Mutation {
            foobar: String!
        }
        "#,
            r#"
        mutation @example {
            foobar
        }
        "#,
        );

        crate::tests::assert_err(
            r#"
        directive @example on FIELD

        type Mutation {
            foobar: String!
        }
        "#,
            r#"
        mutation @example {
            foobar
        }
        "#,
            r#"
        Error: 5.7.2 Directives Are In Valid Locations

          × Directive `example` cannot be used in this location.
           ╭────
         1 │ mutation @example {
           ·          ───┬────
           ·             ╰── Directive `example` is used here but cannot be used in this location.
           ·
         2 │     foobar
         3 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_directives_are_in_valid_locations_subscription() {
        crate::tests::assert_ok(
            r#"
        directive @example on SUBSCRIPTION

        type Subscription {
            foobar: String!
        }
        "#,
            r#"
        subscription @example {
            foobar
        }
        "#,
        );

        crate::tests::assert_err(
            r#"
        directive @example on FIELD

        type Subscription {
            foobar: String!
        }
        "#,
            r#"
        subscription @example {
            foobar
        }
        "#,
            r#"
        Error: 5.7.2 Directives Are In Valid Locations

          × Directive `example` cannot be used in this location.
           ╭────
         1 │ subscription @example {
           ·              ───┬────
           ·                 ╰── Directive `example` is used here but cannot be used in this location.
           ·
         2 │     foobar
         3 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_directives_are_in_valid_locations_field() {
        crate::tests::assert_ok(
            r#"
        directive @example on FIELD

        type Query {
            foobar: String!
        }
        "#,
            r#"
        query {
            foobar @example
        }
        "#,
        );

        crate::tests::assert_err(
            r#"
        directive @example on QUERY

        type Query {
            foobar: String!
        }
        "#,
            r#"
        query {
            foobar @example
        }
        "#,
            r#"
        Error: 5.7.2 Directives Are In Valid Locations
        
          × Directive `example` cannot be used in this location.
           ╭────
         1 │ query {
         2 │     foobar @example
           ·            ───┬────
           ·               ╰── Directive `example` is used here but cannot be used in this location.
           ·
         3 │ }
           ╰────
        "#,
        )
    }

    #[test]
    fn test_directives_are_in_valid_locations_fragment_definition() {
        crate::tests::assert_ok(
            r#"
        directive @example on FRAGMENT_DEFINITION

        type Query {
            foobar: String!
        }
        "#,
            r#"
        fragment foobarFragment on Query @example {
            foobar
        }
        
        query {
            ...foobarFragment
        }
        "#,
        );

        crate::tests::assert_err(
            r#"
        directive @example on QUERY

        type Query {
            foobar: String!
        }
        "#,
            r#"
        fragment foobarFragment on Query @example {
            foobar
        }
        
        query {
            ...foobarFragment
        }
        "#,
            r#"
        Error: 5.7.2 Directives Are In Valid Locations
        
          × Directive `example` cannot be used in this location.
           ╭────
         1 │ fragment foobarFragment on Query @example {
           ·                                  ───┬────
           ·                                     ╰── Directive `example` is used here but cannot be used in this location.
           ·
         2 │     foobar
         3 │ }
         4 │ 
         5 │ query {
         6 │     ...foobarFragment
         7 │ }
           ╰────
        "#,
        );
    }

    #[test]
    fn test_directives_are_in_valid_locations_fragment_spread() {
        crate::tests::assert_ok(
            r#"
        directive @example on FRAGMENT_SPREAD

        type Query {
            foobar: String!
        }
        "#,
            r#"
        fragment foobarFragment on Query {
            foobar
        }
        
        query {
            ...foobarFragment @example
        }
        "#,
        );

        crate::tests::assert_err(
            r#"
        directive @example on QUERY

        type Query {
            foobar: String!
        }
        "#,
            r#"
        fragment foobarFragment on Query {
            foobar
        }
        
        query {
            ...foobarFragment @example
        }
        "#,
            r#"
        Error: 5.7.2 Directives Are In Valid Locations
        
          × Directive `example` cannot be used in this location.
           ╭────
         1 │ fragment foobarFragment on Query {
         2 │     foobar
         3 │ }
         4 │ 
         5 │ query {
         6 │     ...foobarFragment @example
           ·                       ───┬────
           ·                          ╰── Directive `example` is used here but cannot be used in this location.
           ·
         7 │ }
           ╰────
        "#,
        );
    }

    #[test]
    fn test_directives_are_in_valid_locations_inline_fragment() {
        crate::tests::assert_ok(
            r#"
        directive @example on INLINE_FRAGMENT

        type Query {
            foobar: String!
        }
        "#,
            r#"
        query {
            ... on Query @example {
                foobar
            }
        }
        "#,
        );

        crate::tests::assert_err(
            r#"
        directive @example on QUERY

        type Query {
            foobar: String!
        }
        "#,
            r#"
        query {
            ... on Query @example {
                foobar
            }
        }
        "#,
            r#"
            Error: 5.7.2 Directives Are In Valid Locations
            
              × Directive `example` cannot be used in this location.
               ╭────
             1 │ query {
             2 │     ... on Query @example {
               ·                  ───┬────
               ·                     ╰── Directive `example` is used here but cannot be used in this location.
               ·
             3 │         foobar
             4 │     }
             5 │ }
               ╰────
        "#,
        );
    }
}
