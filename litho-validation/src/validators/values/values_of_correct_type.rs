use std::borrow::Cow;

use graphql_parser::query::{Text, Value};
use graphql_parser::schema::{EnumType, Type, TypeDefinition};
use graphql_parser::{query, schema};

use crate::extensions::*;
use crate::{Error, Scope, Traverse, Visitor};

/// # 5.6.1 Values Of Correct Type
/// ## Formal Specification
/// - For each input Value `value` in the document.
///   - Let `type` be the type expected in the position `value` is found.
///   - `value` must be coercible to `type`.
///
/// ## Explanatory Text
/// Literal values must be compatible with the type expected in the position
/// they are found as per the coercion rules defined in the Type System chapter.
///
/// The type expected in a position includes the type defined by the argument a
/// value is provided for, the type defined by an input object field a value is
/// provided for, and the type of a variable definition a default value is
/// provided for.
///
/// The following examples are valid use of value literals:
/// ```graphql
/// fragment goodBooleanArg on Arguments {
///   booleanArgField(booleanArg: true)
/// }
///
/// fragment coercedIntIntoFloatArg on Arguments {
///   # Note: The input coercion rules for Float allow Int literals.
///   floatArgField(floatArg: 123)
/// }
///
/// query goodComplexDefaultValue($search: ComplexInput = { name: "Fido" }) {
///   findDog(complex: $search)
/// }
/// ```
///
/// Non-coercible values (such as a String into an Int) are invalid. The
/// following examples are invalid:
/// ```graphql
/// fragment stringIntoInt on Arguments {
///   intArgField(intArg: "123")
/// }
///
/// query badComplexValue {
///   findDog(complex: { name: 123 })
/// }
/// ```
pub struct ValuesOfCorrectType;

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
        expected: &'v Type<'a, T>,
        actual: ValueType,
    ) -> Result<(), Error<'v, 'a, T>>
    where
        'a: 'v,
        T: Text<'a>,
    {
        Err(Error::IncorrectValueType {
            name,
            span: self.span,
            expected,
            actual,
        })
    }

    fn check_value_type(
        &self,
        name: Cow<'v, str>,
        value: &Value<'a, T>,
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
            Type::NonNullType(inner_ty) => match value {
                Value::Null => {
                    return self.error(name, ty, value.ty());
                }
                _ => {
                    return self.check_value_type(name, value, inner_ty);
                }
            },
            Type::ListType(item_ty) => match value {
                Value::Null => return Ok(()),
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
                _ => return self.error(name, ty, value.ty()),
            },
            Type::NamedType(name) => name,
        };

        match (ty_name.as_ref(), value.ty()) {
            ("Int", ValueType::Int) => {}
            ("Float", ValueType::Int | ValueType::Float) => {}
            ("String", ValueType::String) => {}
            ("Boolean", ValueType::Boolean) => {}
            ("ID", ValueType::String) => {}
            _ => match self.schema.type_definition(ty_name.as_ref()) {
                Some(definition) => {
                    self.check_value_type_definition(name, value, ty, definition)?
                }
                None => self.error(name, ty, value.ty())?,
            },
        }

        Ok(())
    }

    fn check_value_type_definition(
        &self,
        name: Cow<'v, str>,
        value: &Value<'a, T>,
        ty: &'v Type<'a, T>,
        definition: &'v TypeDefinition<'a, T>,
    ) -> Result<(), Error<'v, 'a, T>>
    where
        'a: 'v,
        T: Text<'a>,
    {
        match definition {
            TypeDefinition::Enum(enum_ty) => self.check_value_enum_type(name, value, ty, enum_ty),
            TypeDefinition::InputObject(input_object) => match value {
                Value::Object(object) => {
                    for field in input_object.fields.iter() {
                        if let Some(value) = object.get(field.name.as_ref()) {
                            self.check_value_type(
                                format!("{}.{}", name, field.name.as_ref()).into(),
                                value,
                                &field.value_type,
                            )?;
                        }
                    }

                    Ok(())
                }
                _ => self.error(name, ty, value.ty()),
            },
            TypeDefinition::Interface(_)
            | TypeDefinition::Object(_)
            | TypeDefinition::Scalar(_)
            | TypeDefinition::Union(_) => Ok(()),
        }
    }

    fn check_value_enum_type(
        &self,
        name: Cow<'v, str>,
        value: &Value<'a, T>,
        ty: &'v Type<'a, T>,
        enum_ty: &'v EnumType<'a, T>,
    ) -> Result<(), Error<'v, 'a, T>>
    where
        'a: 'v,
        T: Text<'a>,
    {
        match value {
            Value::Enum(value) if enum_ty.has_value(value.as_ref()) => Ok(()),
            _ => self.error(name, ty, value.ty()),
        }
    }
}

impl<'v, 'a, T> Visitor<'v, 'a, T> for ValuesOfCorrectType
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
            Some(Value::Null) if !variable_definition.var_type.is_required() => {}
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

impl<'v, 'a, T> Traverse<'v, 'a, T> for ValuesOfCorrectType
where
    'a: 'v,
    T: Text<'a>,
{
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_values_of_correct_types_157() {
        crate::tests::assert_ok(
            r#"
        type Arguments {
            booleanArgField(booleanArg: Boolean!): Boolean!
            floatArgField(floatArg: Float!): Float!
        }

        type Dog {
            name: String!
        }

        input ComplexInput {
            name: String!
        }

        type Query {
            arguments: Arguments!
            findDog(complex: ComplexInput!): Dog!
        }
        "#,
            r#"
        fragment goodBooleanArg on Arguments {
            booleanArgField(booleanArg: true)
        }

        fragment coercedIntIntoFloatArg on Arguments {
            # Note: The input coercion rules for Float allow Int literals.
            floatArgField(floatArg: 123)
        }

        query goodComplexDefaultValue($search: ComplexInput = { name: "Fido" }) {
            arguments {
                ... goodBooleanArg
                ... coercedIntIntoFloatArg
            }

            findDog(complex: $search) {
                name
            }
        }
        "#,
        )
    }

    #[test]
    fn test_values_of_correct_types_158() {
        crate::tests::assert_err(
            r#"
        type Arguments {
            intArgField(intArg: Int!): Boolean!
        }

        input ComplexInput {
            name: Boolean!
        }

        type Query {
            arguments: Arguments!
            findDog(complex: ComplexInput!): Boolean!
        }
        "#,
            r#"
        fragment stringIntoInt on Arguments {
            intArgField(intArg: "123")
        }

        query badComplexValue {
            arguments {
                ... stringIntoInt
            }
            findDog(complex: { name: 123 })
        }
        "#,
            r#"
        Error: 5.6.1 Values Of Correct Type
        
          × Value provided for `intArg` has incorrect type.
            ╭────
          1 │ fragment stringIntoInt on Arguments {
          2 │     intArgField(intArg: "123")
            ·     ─────┬─────
            ·          ╰── Value provided for `intArg` here is type `STRING` but expected `Int`.
            ·
          3 │ }
          4 │ 
          5 │ query badComplexValue {
          6 │     arguments {
          7 │         ... stringIntoInt
          8 │     }
          9 │     findDog(complex: { name: 123 })
         10 │ }
            ╰────
        
        
        Error: 5.6.1 Values Of Correct Type
        
          × Value provided for `complex.name` has incorrect type.
            ╭────
          1 │ fragment stringIntoInt on Arguments {
          2 │     intArgField(intArg: "123")
          3 │ }
          4 │ 
          5 │ query badComplexValue {
          6 │     arguments {
          7 │         ... stringIntoInt
          8 │     }
          9 │     findDog(complex: { name: 123 })
            ·     ───┬───
            ·        ╰── Value provided for `complex.name` here is type `INT` but expected `Boolean`.
            ·
         10 │ }
            ╰────
        "#,
        )
    }

    #[test]
    fn test_values_of_correct_types_null() {
        crate::tests::assert_err(
            r#"
        input ComplexInput {
            name: Boolean!
        }

        type Query {
            findDog(complex: [ComplexInput!]!): Boolean!
        }
        "#,
            r#"
        query badComplexValue {
            a: findDog(complex: [null])
            b: findDog(complex: null)
        }
        "#,
            r#"
        Error: 5.4.2.1 Required Arguments
        
          × Required argument `complex` is missing for field `findDog` of type `Query`.
           ╭────
         1 │ query badComplexValue {
         2 │     a: findDog(complex: [null])
         3 │     b: findDog(complex: null)
           ·     ───┬───
           ·        ╰── Argument `complex` is required but missing for field `findDog` of type `Query`.
           ·
         4 │ }
           ╰────
        
        
        Error: 5.6.1 Values Of Correct Type
        
          × Value provided for `complex[0]` has incorrect type.
           ╭────
         1 │ query badComplexValue {
         2 │     a: findDog(complex: [null])
           ·     ───┬───
           ·        ╰── Value provided for `complex[0]` here is type `NULL` but expected `ComplexInput!`.
           ·
         3 │     b: findDog(complex: null)
         4 │ }
           ╰────
        
        
        Error: 5.6.1 Values Of Correct Type
        
          × Value provided for `complex` has incorrect type.
           ╭────
         1 │ query badComplexValue {
         2 │     a: findDog(complex: [null])
         3 │     b: findDog(complex: null)
           ·     ───┬───
           ·        ╰── Value provided for `complex` here is type `NULL` but expected `[ComplexInput!]!`.
           ·
         4 │ }
           ╰────
        "#,
        );
    }
}
