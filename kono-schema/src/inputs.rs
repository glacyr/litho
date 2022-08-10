use graphql_parser::schema;

use super::{Emit, Fields, Type};

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

/// Value that is passed to a field or directive.
pub struct InputValue {
    name: String,
    description: Option<String>,
    ty: Type,
}

impl InputValue {
    /// Returns a new input value with the given name and type.
    pub fn new(name: &str, ty: Type) -> InputValue {
        InputValue {
            name: name.to_owned(),
            description: None,
            ty,
        }
    }

    /// Adds the given description to this input value and returns the result.
    pub fn description(mut self, description: &str) -> InputValue {
        self.description = Some(description.to_owned());
        self
    }
}

impl Emit for InputValue {
    type Target = schema::InputValue<'static, String>;

    fn emit(self) -> Self::Target {
        schema::InputValue {
            name: self.name,
            value_type: self.ty.emit(),
            description: self.description,
            default_value: None,
            directives: vec![],
            position: Default::default(),
        }
    }
}
