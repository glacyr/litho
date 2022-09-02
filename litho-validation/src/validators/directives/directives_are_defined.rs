use graphql_parser::query::Text;
use graphql_parser::schema;

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.7.1 Directives Are Defined
/// ## Formal Specification
/// - For every `directive` in a document.
/// - Let `directiveName` be the name of `directive`.
/// - Let `directiveDefinition` be the directive named `directiveName`.
/// - `directiveDefinition` must exist.
pub struct DirectivesAreDefined;

impl<'v, 'a, T> Visitor<'v, 'a, T> for DirectivesAreDefined
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator = Vec<Error<'v, 'a, T>>;

    fn visit_directive(
        &self,
        directive: &'v schema::Directive<'a, T>,
        schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        if schema.directive_definition(&directive.name).is_none() {
            accumulator.push(Error::UndefinedDirective {
                name: directive.name.as_ref(),
                span: directive.span(),
            })
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for DirectivesAreDefined
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_undefined_directive() {
        crate::tests::assert_err(
            r#"
        type Query {
            foobar: String!
        }
        "#,
            r#"
        {
            foobar @example
        }
        "#,
            r#"
        Error: 5.7.1 Directives Are Defined

          × Directive `example` does not exist.
           ╭────
         1 │ {
         2 │     foobar @example
           ·            ───┬────
           ·               ╰── Directive `example` is referenced here but does not exist.
           ·
         3 │ }
           ╰────
        "#,
        )
    }
}
