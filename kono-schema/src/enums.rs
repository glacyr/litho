use graphql_parser::schema;
use heck::ToLowerCamelCase;

use super::{Directive, Emit, Field, Fields, Item, ItemType, Type};

/// Representation of an enum with zero or more variants with optionally named
/// or unnamed fields.
///
/// ### Enums with Named Fields
/// ```rust ignore
/// #[derive(Kono)]
/// pub enum Animal {
///     Cat {
///         name: String,
///     },
///     Dog,
/// }
/// ```
/// Translates into:
/// ```graphql
/// type AnimalCat {
///     name: String!
/// }
///
/// type Animal {
///     cat: AnimalCat
///     dog: Boolean
/// }
/// ```
///
/// ### Enums with Unnamed Fields
/// ```rust ignore
/// #[derive(Kono)]
/// pub enum Animal {
///     Cat(String, u32),
///     Dog,
/// }
/// ```
/// Translates into:
/// ```graphql
/// type AnimalCat {
///     _0: String!
///     _1: Int!
/// }
///
/// type Animal {
///     cat: AnimalCat
///     dog: Boolean
/// }
/// ```
///
/// ### Unions
/// ```rust ignore
/// #[derive(Kono)]
/// pub enum Animal {
///     Cat(Cat),
///     Dog(Dog),
/// }
/// ```
/// Translates into:
/// ```graphql
/// union Animal = Cat | Dog
/// ```
///
/// ### Unions with Scalars
/// ```rust ignore
/// #[derive(Kono)]
/// pub enum Value {
///     String(String),
///     Complex(Complex),
/// }
/// ```
/// Translates into:
/// ```graphql
/// type Value {
///     string: String
///     complex: Complex
/// }
/// ```
///
/// ### Generic Unions
/// ```rust ignore
/// #[derive(Kono)]
/// pub enum Either<A, B> {
///     A(A),
///     B(B),
/// }
/// ```
/// Translates into:
/// ```graphql
/// union EitherAB = A | B
/// ```
#[derive(Default)]
pub struct ItemEnum {
    name: String,
    description: Option<String>,
    variants: Vec<Variant>,
}

impl ItemEnum {
    /// Returns a new enum item with the given name.
    pub fn new(name: &str) -> ItemEnum {
        ItemEnum {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Adds the given description to this enum item and returns the result.
    pub fn description(mut self, description: &str) -> ItemEnum {
        self.description.replace(description.to_owned());
        self
    }

    /// Adds the given variant to this enum item and returns the result.
    pub fn variant(mut self, variant: Variant) -> ItemEnum {
        self.variants.push(variant);
        self
    }

    fn emit_as_union(self) -> Vec<schema::TypeDefinition<'static, String>> {
        vec![schema::TypeDefinition::Union(schema::UnionType {
            name: self.name,
            description: self.description,
            types: self
                .variants
                .into_iter()
                .map(|variant| variant.name)
                .collect(),
            directives: vec![],
            position: Default::default(),
        })]
    }

    fn emit_as_enum(self) -> Vec<schema::TypeDefinition<'static, String>> {
        vec![schema::TypeDefinition::Enum(schema::EnumType {
            name: self.name,
            description: self.description,
            values: self.variants.into_iter().map(Emit::emit).collect(),
            directives: vec![],
            position: Default::default(),
        })]
    }

    fn emit_as_type(self) -> Vec<schema::TypeDefinition<'static, String>> {
        let name = self.name;

        let (definitions, fields): (Vec<_>, Vec<_>) = self
            .variants
            .into_iter()
            .map(|variant| {
                let (definition, ty) = match variant.fields {
                    Fields::Unit => (None, Type::Named("Boolean".to_owned())),
                    Fields::Unnamed(mut unnamed) if unnamed.len() == 1 => {
                        (None, unnamed.remove(0).ty)
                    }
                    fields => (
                        Some(ItemType::new(&format!("{}{}", name, variant.name)).fields(fields)),
                        Type::Named(name.to_owned() + &variant.name),
                    ),
                };

                (
                    definition,
                    Field::new(
                        Some(&variant.name.to_lower_camel_case()),
                        Type::Optional(Box::new(ty)),
                    ),
                )
            })
            .unzip();

        let mut definitions = definitions.into_iter().flatten().collect::<Vec<_>>();
        let fields = fields.into_iter().collect::<Vec<_>>();

        definitions.push(
            ItemType::new(&name)
                .description(self.description.as_deref())
                .directive(Directive::new("oneOf"))
                .fields(Fields::Named(fields)),
        );

        definitions.into_iter().map(Emit::emit).flatten().collect()
    }
}

impl Emit for ItemEnum {
    type Target = Vec<schema::TypeDefinition<'static, String>>;

    fn emit(self) -> Self::Target {
        if !self.variants.is_empty() {
            if self.variants.iter().all(|variant| variant.union()) {
                return self.emit_as_union();
            }

            if self
                .variants
                .iter()
                .all(|variant| matches!(variant.fields, Fields::Unit))
            {
                return self.emit_as_enum();
            }
        }

        self.emit_as_type()
    }
}

impl Into<Item> for ItemEnum {
    fn into(self) -> Item {
        Item::Enum(self)
    }
}

/// Individual enum value with optional associated fields.
#[derive(Default)]
pub struct Variant {
    name: String,
    description: Option<String>,
    fields: Fields,
}

impl Variant {
    /// Returns a new enum variant with the given name.
    pub fn new(name: &str) -> Variant {
        Variant {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Adds the given description to this enum variant and returns the result.
    pub fn description(mut self, description: &str) -> Variant {
        self.description = Some(description.to_owned());
        self
    }

    /// Adds the given fields to this enum variant and returns the result.
    pub fn fields(mut self, fields: Fields) -> Variant {
        self.fields = fields.flatten();
        self
    }

    /// Returns a boolean that indicates if this variant may be part of a union
    /// or is definitively not. For a variant to qualify as a union, it should
    /// have one field, its type must not be scalar and its type's name should
    /// be identical to the variant's name (e.g. `Cat(Cat)`).
    pub fn union(&self) -> bool {
        match &self.fields {
            Fields::Unnamed(fields) => match fields.as_slice() {
                [Field {
                    ty: Type::Named(field),
                    ..
                }] => field == &self.name,
                _ => false,
            },
            _ => false,
        }
    }
}

impl Emit for Variant {
    type Target = schema::EnumValue<'static, String>;

    fn emit(self) -> Self::Target {
        assert!(matches!(self.fields, Fields::Unit));

        schema::EnumValue {
            name: self.name,
            description: self.description,
            directives: vec![],
            position: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, Fields, ItemEnum, Type, Variant};

    use crate::tests::test_eq;

    #[test]
    fn test_regular_enums() {
        test_eq(
            ItemEnum::new("Animal")
                .description("Enum that represents a particular species.")
                .variant(Variant::new("Cat"))
                .variant(Variant::new("Dog"))
                .variant(Variant::new("Horse")),
            r#"
            "Enum that represents a particular species."
			enum Animal {
			  Cat
			  Dog
			  Horse
			}
            "#,
        );
    }

    #[test]
    fn test_enums_without_variants() {
        test_eq(
            ItemEnum::new("Animal"),
            r#"
            type Animal @oneOf {
              __typename: String!
            }
            "#,
        );
    }

    #[test]
    fn test_enums_with_named_fields() {
        test_eq(
            ItemEnum::new("Animal")
                .description("Enum that represents a particular species.")
                .variant(Variant::new("Cat").fields(Fields::Named(vec![Field::new(
                    Some("name"),
                    Type::Scalar("String".to_owned()),
                )])))
                .variant(Variant::new("Dog")),
            r#"
			type AnimalCat {
			  __typename: String!
			  name: String!
			}

			"Enum that represents a particular species."
			type Animal @oneOf {
			  __typename: String!
			  cat: AnimalCat
			  dog: Boolean
			}
            "#,
        );
    }

    #[test]
    fn test_enums_with_unnamed_fields() {
        test_eq(
            ItemEnum::new("Animal")
                .description("Enum that represents a particular species.")
                .variant(Variant::new("Cat").fields(Fields::Unnamed(vec![
                    Field::new(None, Type::Scalar("String".to_owned())),
                    Field::new(None, Type::Scalar("Int".to_owned())),
                ])))
                .variant(Variant::new("Dog")),
            r#"
			type AnimalCat {
			  __typename: String!
			  _0: String!
			  _1: Int!
			}

			"Enum that represents a particular species."
			type Animal @oneOf {
			  __typename: String!
			  cat: AnimalCat
			  dog: Boolean
			}
            "#,
        );
    }

    #[test]
    fn test_unions() {
        test_eq(
            ItemEnum::new("Animal")
                .description("Union that represents a particular species.")
                .variant(Variant::new("Cat").fields(Fields::Unnamed(vec![Field::new(
                    None,
                    Type::Named("Cat".to_owned()),
                )])))
                .variant(Variant::new("Dog").fields(Fields::Unnamed(vec![Field::new(
                    None,
                    Type::Named("Dog".to_owned()),
                )]))),
            r#"
			"Union that represents a particular species."
			union Animal = Cat | Dog
            "#,
        );
    }

    #[test]
    fn test_unions_with_scalars() {
        test_eq(
            ItemEnum::new("Value")
                .variant(
                    Variant::new("String").fields(Fields::Unnamed(vec![Field::new(
                        None,
                        Type::Scalar("String".to_owned()),
                    )])),
                )
                .variant(
                    Variant::new("Complex").fields(Fields::Unnamed(vec![Field::new(
                        None,
                        Type::Named("Complex".to_owned()),
                    )])),
                ),
            r#"
			type Value @oneOf {
			  __typename: String!
			  string: String
			  complex: Complex
			}
            "#,
        );
    }
}
