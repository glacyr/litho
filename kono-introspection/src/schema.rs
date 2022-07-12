use std::marker::PhantomData;
use std::rc::Rc;

use graphql_parser::schema;
use kono_macros::kono;

use super::{Type, TypeDefinitionExt};

pub struct Schema<C = ()> {
    _context: PhantomData<C>,
    schema: Rc<schema::Document<'static, String>>,
}

impl<C> Schema<C> {
    pub fn new(schema: schema::Document<'static, String>) -> Schema<C> {
        Schema {
            _context: PhantomData,
            schema: Rc::new(schema),
        }
    }
}

#[kono]
impl<C> Aspect for Schema<C>
where
    C: 'static,
{
    type Context = C;
    type Environment = schema::Document<'static, String>;

    #[kono(query, rename = "__schema")]
    fn schema(environment: &schema::Document<'static, String>) -> Schema<C> {
        Schema::new(environment.to_owned())
    }

    fn query_type(&self) -> Type<C> {
        let definition = self.schema.type_definition_or_default("Query");

        Type::new(&self.schema, &definition)
    }

    fn mutation_type(&self) -> Type<C> {
        let definition = self.schema.type_definition_or_default("Mutation");

        Type::new(&self.schema, &definition)
    }

    fn subscription_type(&self) -> Type<C> {
        let definition = self.schema.type_definition_or_default("Subscription");

        Type::new(&self.schema, &definition)
    }
}

pub trait SchemaExt<'a, T>
where
    T: schema::Text<'a>,
    T::Value: PartialEq<str>,
{
    fn type_definition(&self, name: &str) -> Option<&schema::TypeDefinition<'a, T>>;
    fn type_definition_or_default(&self, name: &str) -> schema::TypeDefinition<'a, T>;
}

impl<'a, T> SchemaExt<'a, T> for schema::Document<'a, T>
where
    T: schema::Text<'a> + Clone,
    T::Value: PartialEq<str> + for<'b> From<&'b str>,
{
    fn type_definition(&self, name: &str) -> Option<&schema::TypeDefinition<'a, T>> {
        self.definitions
            .iter()
            .flat_map(|definition| match definition {
                schema::Definition::TypeDefinition(ty) if ty.name() == name => Some(ty),
                _ => None,
            })
            .next()
    }

    fn type_definition_or_default(&self, name: &str) -> schema::TypeDefinition<'a, T> {
        self.type_definition(name)
            .cloned()
            .unwrap_or(schema::TypeDefinition::Object(schema::ObjectType::new(
                name.into(),
            )))
    }
}
