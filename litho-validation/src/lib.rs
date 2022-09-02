use graphql_parser::query::Text;
use graphql_parser::{query, schema};

pub mod diagnostics;
mod error;
mod extensions;
mod validators;
mod walk;

pub use error::Error;
pub use validators::*;
pub use walk::{Scope, Traverse, Visitor};

pub fn validate<'a, 'b, T>(
    schema: &'a schema::Document<'b, T>,
    query: &'a query::Document<'b, T>,
) -> Result<(), Vec<Error<'a, 'b, T>>>
where
    'a: 'b,
    T: Text<'a>,
{
    let mut errors = vec![];

    ExecutableDefinitions.traverse(query, schema, &mut errors);
    OperationNameUniqueness.traverse(query, schema, &mut errors);
    LoneAnonymousOperation.traverse(query, schema, &mut errors);
    FieldSelections.traverse(query, schema, &mut errors);
    LeafFieldSelections.traverse(query, schema, &mut errors);
    ArgumentNames.traverse(query, schema, &mut errors);
    ArgumentUniqueness.traverse(query, schema, &mut errors);
    RequiredArguments.traverse(query, schema, &mut errors);
    FragmentNameUniqueness.traverse(query, schema, &mut errors);
    FragmentSpreadTypeExistence.traverse(query, schema, &mut errors);
    FragmentsOnCompositeTypes.traverse(query, schema, &mut errors);

    if errors.is_empty() {
        return Ok(());
    }

    Err(errors)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
