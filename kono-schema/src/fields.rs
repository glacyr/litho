use std::iter::once;

use graphql_parser::schema;

use super::{Emit, Type};

/// Individual (optionally named) field of an input type, object type or enum.
pub struct Field {
    name: Option<String>,
    description: Option<String>,
    pub(crate) ty: Type,
}

impl Field {
    /// Returns a new field with the given type and (optionally) the given name.
    pub fn new(name: Option<&str>, ty: Type) -> Field {
        Field {
            name: name.map(ToOwned::to_owned),
            description: None,
            ty,
        }
    }

    /// Adds the given description to this field and returns the result.
    pub fn description(mut self, description: &str) -> Field {
        self.description = Some(description.to_owned());
        self
    }

    fn name_or_index(mut self, index: usize) -> Field {
        self.name = Some(self.name.unwrap_or(format!("_{}", index)));
        self
    }
}

impl Emit for Field {
    type Target = schema::Field<'static, String>;

    fn emit(self) -> Self::Target {
        schema::Field {
            name: self.name.unwrap_or("".to_owned()),
            description: self.description,
            arguments: vec![],
            field_type: self.ty.emit(),
            directives: vec![],
            position: Default::default(),
        }
    }
}

/// Fields of an input ([`ItemInput::fields`](super::ItemInput::fields)), object
/// ([`ItemType::fields`](super::ItemType::fields)) or enum item
/// ([`Variant::fields`](super::Variant::fields)).
pub enum Fields {
    /// Represents a type without any fields (i.e. a unit struct in Rust).
    Unit,

    /// Represents a type with named fields (i.e. a regular struct).
    Named(Vec<Field>),

    /// Represents a type with unnamed fields (i.e. a tuple).
    Unnamed(Vec<Field>),
}

impl Fields {
    /// Returns [`Fields::Unit`] if the receiver is empty.
    pub fn flatten(self) -> Fields {
        match self {
            Fields::Unit => self,
            Fields::Named(fields) | Fields::Unnamed(fields) if fields.is_empty() => Fields::Unit,
            fields => fields,
        }
    }
}

impl Default for Fields {
    fn default() -> Self {
        Fields::Unit
    }
}

impl Emit for Fields {
    type Target = Vec<schema::Field<'static, String>>;

    fn emit(self) -> Self::Target {
        let fields = match self {
            Fields::Unit => vec![].into_iter(),
            Fields::Named(fields) | Fields::Unnamed(fields) => fields.into_iter(),
        }
        .enumerate()
        .map(|(index, field)| field.name_or_index(index));

        once(Field::new(
            Some("__typename"),
            Type::Named("String".to_owned()),
        ))
        .chain(fields)
        .map(Emit::emit)
        .collect()
    }
}
