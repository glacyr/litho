use graphql_parser::schema;

use super::{Directive, Emit, Fields, Item};

/// Representation of an object type.
///
/// ### Types without Fields
/// ```rust ignore
/// #[derive(Kono)]
/// pub struct Unit;
/// ```
/// Translates into:
/// ```graphql
/// type Unit {
///     __typename: String
/// }
/// ```
///
/// ### Types with Named Fields
/// ```rust ignore
/// #[derive(Kono)]
/// pub struct Cat {
///     name: String,
/// }
/// ```
/// Translates into:
/// ```graphql
/// type Cat {
///     __typename: String!
///     name: String!
/// }
/// ```
///
/// ### Types with Unnamed Fields
/// ```rust ignore
/// #[derive(Kono)]
/// pub struct Cat(String, u32);
/// ```
/// Translates into:
/// ```graphql
/// type Cat {
///     __typename: String!
///     _0: String!
///     _1: Int!
/// }
/// ```
#[derive(Default)]
pub struct ItemType {
    name: String,
    description: Option<String>,
    directives: Vec<Directive>,
    fields: Fields,
}

impl ItemType {
    /// Returns a new type item with the given name.
    pub fn new(name: &str) -> ItemType {
        ItemType {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Adds the given description to this type and returns the result.
    pub fn description<'a>(mut self, description: impl Into<Option<&'a str>>) -> ItemType {
        self.description = description.into().map(ToOwned::to_owned);
        self
    }

    /// Adds the given directive to this type and returns the result.
    pub fn directive(mut self, directive: Directive) -> ItemType {
        self.directives.push(directive);
        self
    }

    /// Adds the given fields to this type and returns the result.
    pub fn fields(mut self, fields: Fields) -> ItemType {
        self.fields = fields.flatten();
        self
    }
}

impl Emit for ItemType {
    type Target = Vec<schema::TypeDefinition<'static, String>>;

    fn emit(self) -> Self::Target {
        vec![schema::TypeDefinition::Object(schema::ObjectType {
            name: self.name,
            description: self.description,
            fields: self.fields.emit(),
            implements_interfaces: vec![],
            directives: self.directives.into_iter().map(Emit::emit).collect(),
            position: Default::default(),
        })]
    }
}

impl Into<Item> for ItemType {
    fn into(self) -> Item {
        Item::Type(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Fields, ItemType};

    use crate::tests::test_eq;
    use crate::{Field, Type};

    #[test]
    fn test_types_without_fields() {
        test_eq(
            ItemType::new("Unit"),
            r#"
			type Unit {
			  __typename: String!
			}
			"#,
        );
    }

    #[test]
    fn test_types_with_named_fields() {
        test_eq(
            ItemType::new("Cat").fields(Fields::Named(vec![Field::new(
                Some("name"),
                Type::Scalar("String".to_owned()),
            )])),
            r#"
			type Cat {
			  __typename: String!
			  name: String!
			}
			"#,
        );
    }

    #[test]
    fn test_types_with_unnamed_fields() {
        test_eq(
            ItemType::new("Cat").fields(Fields::Unnamed(vec![
                Field::new(None, Type::Scalar("String".to_owned())),
                Field::new(None, Type::Scalar("Int".to_owned())),
            ])),
            r#"
			type Cat {
			  __typename: String!
			  _0: String!
			  _1: Int!
			}
			"#,
        );
    }
}
