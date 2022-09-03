use std::borrow::Cow;

use graphql_parser::query::{Text, Value};
use graphql_parser::schema::Type;
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.6.2 Input Object Field Names
/// ## Formal Specification
/// - For each Input Object Field `inputField` in the document
/// - Let `inputFieldName` be the Name of `inputField`.
/// - Let `inputFieldDefinition` be the input field definition provided by the
///   parent input object type named `inputFieldName`.
/// - `inputFieldDefinition` must exist.
///
/// ## Explanatory Text
/// Every input field provided in an input object value must be defined in the
/// set of possible fields of that input object's expected type.
///
/// For example the following example input object is valid:
/// ```graphql
/// {
///   findDog(complex: { name: "Fido" })
/// }
/// ```
///
/// While the following example input-object uses a field `favoriteCookieFlavor`
/// which is not defined on the expected type:
/// ```graphql
/// {
///   findDog(complex: { favoriteCookieFlavor: "Bacon" })
/// }
/// ```
pub struct InputObjectFieldNames;

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
        Err(Error::UndefinedInputObjectField {
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
                for (key, _) in object.iter() {
                    match definition.input_value(key) {
                        Some(field) => {
                            self.check_value_type(
                                format!("{}.{}", name, field.name.as_ref()).into(),
                                value,
                                &field.value_type,
                            )?;
                        }
                        None => return self.error(name, ty, key.as_ref()),
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for InputObjectFieldNames
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

impl<'v, 'a, T> Traverse<'v, 'a, T> for InputObjectFieldNames
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    const SCHEMA: &'static str = r#"
    input ComplexInput {
        name: String
    }

    type Query {
        findDog(complex: ComplexInput!): Boolean!
    }
    "#;

    #[test]
    fn test_input_object_field_names_159() {
        crate::tests::assert_ok(
            SCHEMA,
            r#"
        {
            findDog(complex: { name: "Fido" })
        }
        "#,
        )
    }

    #[test]
    fn test_input_object_field_names_160() {
        crate::tests::assert_err(
            SCHEMA,
            r#"
        {
            findDog(complex: { favoriteCookieFlavor: "Bacon" })
        }
        "#,
            r#"
        Error: 5.6.1 Values Of Correct Type
        
          × Field `favoriteCookieFlavor` does not exist for input type `ComplexInput`.
           ╭────
         1 │ {
         2 │     findDog(complex: { favoriteCookieFlavor: "Bacon" })
           ·     ───┬───
           ·        ╰── Input `complex` resolves to type `ComplexInput` here but does not have field `favoriteCookieFlavor`.
           ·
         3 │ }
           ╰────
        "#,
        )
    }
}
