use std::borrow::Cow;

use graphql_parser::query::{Text, Value};
use graphql_parser::schema::Type;
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.6.4 Input Object Required Fields
/// ## Formal Specification
/// - For each Input Object in the document.
///   - Let `fields` be the fields provided by that Input Object.
///   - Let `fieldDefinitions` be the set of input field definitions of that
///     Input Object.
/// - For each `fieldDefinition` in `fieldDefinitions`:
///   - Let `type` be the expected type of `fieldDefinition`.
///   - Let `defaultValue` be the default value of `fieldDefinition`.
///   - If `type` is Non-Null and `defaultValue does not exist:
///     - Let `fieldName` be the name of `fieldDefinition`.
///     - Let `field` be the input field in `fields` named `fieldName`
///     - `field` must exist.
///     - Let `value` be the value of `field`.
///     - `value` must not be the `null` literal.
///
/// ## Explanatory Text
/// Input object fields may be required. Much like a field may have required
/// arguments, an input objects may have required fields. An input field is
/// required if it has a non-null type and does not have a default value.
/// Otherwise, the input object field is optional.
pub struct InputObjectRequiredFields;

struct TypeChecker<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    span: Span,
    schema: &'v schema::Document<'a, T>,
}

impl<'v, 'a, T> TypeChecker<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    fn new(span: Span, schema: &'v schema::Document<'a, T>) -> TypeChecker<'v, 'a, T> {
        TypeChecker { span, schema }
    }

    fn error(
        &self,
        name: Cow<'v, str>,
        ty: &'v Type<'a, T>,
        field: &'v str,
    ) -> Result<(), Error<'v, 'a, T>>
    where
        'a: 'v,
        T: Text<'a>,
    {
        Err(Error::MissingRequiredInputObjectField {
            name,
            span: self.span,
            ty,
            field,
        })
    }

    fn check_value_type(
        &self,
        name: Cow<'v, str>,
        value: &'v Value<'a, T>,
        ty: &'v Type<'a, T>,
    ) -> Result<(), Error<'v, 'a, T>>
    where
        'a: 'v,
        T: Text<'a>,
    {
        if let Value::Variable(_) = value {
            return Ok(());
        }

        let ty_name = match ty {
            Type::NonNullType(inner_ty) => {
                return match value {
                    Value::Null => Ok(()),
                    _ => self.check_value_type(name, value, inner_ty),
                }
            }
            Type::ListType(item_ty) => match value {
                Value::List(items) => {
                    for (index, item) in items.iter().enumerate() {
                        self.check_value_type(
                            format!("{}[{}]", name, index).into(),
                            item,
                            item_ty,
                        )?;
                    }

                    return Ok(());
                }
                _ => return Ok(()),
            },
            Type::NamedType(name) => name,
        };

        match (self.schema.type_definition(ty_name.as_ref()), value) {
            (Some(definition), Value::Object(object)) => {
                for field in definition.input_values() {
                    match (object.get(field.name.as_ref()), &field.value_type) {
                        (Some(value), ty) => {
                            self.check_value_type(
                                format!("{}.{}", name, field.name.as_ref()).into(),
                                value,
                                ty,
                            )?;
                        }
                        (None, Type::NonNullType(_)) if field.default_value.is_null() => {
                            return self.error(name, ty, field.name.as_ref());
                        }
                        (None, _) => {}
                    };
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for InputObjectRequiredFields
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
        let def = schema.directive_definition(&directive.name);

        let def = match def {
            Some(def) => def,
            None => return,
        };

        let checker = TypeChecker::new(directive.span(), schema);

        accumulator.extend(
            directive
                .arguments
                .iter()
                .map(|(name, value)| {
                    let expected = match def.argument(name.as_ref()) {
                        Some(value) => &value.value_type,
                        _ => return None,
                    };

                    checker
                        .check_value_type(name.as_ref().into(), value, expected)
                        .err()
                })
                .flatten(),
        )
    }

    fn visit_field(
        &self,
        field: &'v query::Field<'a, T>,
        schema: &'v schema::Document<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let def = schema
            .type_definition(scope.ty())
            .and_then(|ty| ty.field(&field.name));

        let def = match def {
            Some(def) => def,
            None => return,
        };

        let checker = TypeChecker::new(field.span(), schema);

        accumulator.extend(
            field
                .arguments
                .iter()
                .map(|(name, value)| {
                    let expected = match def.argument(name.as_ref()) {
                        Some(value) => &value.value_type,
                        _ => return None,
                    };

                    checker
                        .check_value_type(name.as_ref().into(), value, expected)
                        .err()
                })
                .flatten(),
        )
    }

    fn visit_variable_definition(
        &self,
        variable_definition: &'v query::VariableDefinition<'a, T>,
        schema: &'v schema::Document<'a, T>,
        _scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        match variable_definition.default_value.as_ref() {
            Some(value) => {
                accumulator.extend(
                    TypeChecker::new(variable_definition.span(), schema)
                        .check_value_type(
                            variable_definition.name.as_ref().into(),
                            value,
                            &variable_definition.var_type,
                        )
                        .err()
                        .into_iter(),
                );
            }
            None => {}
        }
    }
}

impl<'v, 'a, T> Traverse<'v, 'a, T> for InputObjectRequiredFields
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_input_object_required_fields() {
        crate::tests::assert_err(
            r#"
        input ComplexNestedInput {
            foo: String!
            bar: String!
        }

        input ComplexInput {
            nested: ComplexNestedInput!
        }

        type Query {
            foobar(input: ComplexInput!): Boolean!
        }
        "#,
            r#"
        query {
            foobar(input: {
                nested: {
                    foo: "foo"
                }
            })
        }
        "#,
            r#"
        Error: 5.6.4 Input Object Required Fields
        
          × Field `bar` is required for input type `ComplexNestedInput`.
           ╭────
         1 │ query {
         2 │     foobar(input: {
           ·     ──┬───
           ·       ╰── Input `input.nested` resolves to type `ComplexNestedInput` here but requires field `bar`.
           ·
         3 │         nested: {
         4 │             foo: "foo"
         5 │         }
         6 │     })
         7 │ }
           ╰────
        "#,
        )
    }
}
