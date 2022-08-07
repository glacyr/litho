use graphql_parser::schema;

use super::{Emit, Fields};

/// Representation of an input type with zero or more fields.
#[derive(Default)]
pub struct ItemInput {
    name: String,
    description: Option<String>,
    fields: Fields,
}

impl ItemInput {
    /// Returns a new input item with the given name.
    pub fn new(name: &str) -> ItemInput {
        ItemInput {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Adds the given description to this input item and returns the result.
    pub fn description<'a>(mut self, description: impl Into<Option<&'a str>>) -> ItemInput {
        self.description = description.into().map(ToOwned::to_owned);
        self
    }

    /// Adds the given fields to this input item and returns the result.
    pub fn fields(mut self, fields: Fields) -> ItemInput {
        self.fields = fields.flatten();
        self
    }
}

impl Emit for ItemInput {
    type Target = Vec<schema::TypeDefinition<'static, String>>;

    fn emit(self) -> Self::Target {
        vec![schema::TypeDefinition::InputObject(
            schema::InputObjectType {
                name: self.name,
                description: self.description,
                fields: vec![],
                directives: vec![],
                position: Default::default(),
            },
        )]
    }
}
