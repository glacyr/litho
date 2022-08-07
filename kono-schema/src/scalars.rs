use graphql_parser::schema;

use super::Emit;

/// Representation of a custom scalar.
#[derive(Default)]
pub struct ItemScalar {
    name: String,
    description: Option<String>,
}

impl ItemScalar {
    /// Returns a new scalar item with the given name.
    pub fn new(name: &str) -> ItemScalar {
        ItemScalar {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Adds the given description to this scalar item and returns the result.
    pub fn description(mut self, description: &str) -> ItemScalar {
        self.description = Some(description.to_owned());
        self
    }
}

impl Emit for ItemScalar {
    type Target = Vec<schema::TypeDefinition<'static, String>>;

    fn emit(self) -> Self::Target {
        vec![schema::TypeDefinition::Scalar(schema::ScalarType {
            name: self.name,
            description: self.description,
            directives: vec![],
            position: Default::default(),
        })]
    }
}

#[cfg(test)]
mod tests {
    use super::ItemScalar;

    use crate::tests::test_eq;

    #[test]
    fn test_scalar() {
        test_eq(
            ItemScalar::new("DateTime").description("Scalar that represents a point in time."),
            r#"
            "Scalar that represents a point in time."
            scalar DateTime
            "#,
        );
    }
}
