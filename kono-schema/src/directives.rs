use graphql_parser::schema;

use super::Emit;

/// Decorations of the GraphQL schema. Currently, these directives can only be
/// used on types ([`ItemType::directive`](super::ItemType::directive)).
///
/// ```graphql
/// type Example @oneOf { ... }
/// ```
pub struct Directive {
    name: String,
}

impl Directive {
    /// Returns a new directive with the given name.
    pub fn new(name: &str) -> Directive {
        Directive {
            name: name.to_owned(),
        }
    }
}

impl Emit for Directive {
    type Target = schema::Directive<'static, String>;

    fn emit(self) -> Self::Target {
        schema::Directive {
            name: self.name,
            arguments: vec![],
            position: Default::default(),
        }
    }
}
