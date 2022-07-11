use std::marker::PhantomData;

use graphql_parser::schema::{Definition, Document, TypeDefinition};

use kono_macros::kono;

mod ext;

use ext::TypeDefinitionExt;

pub struct Introspection<C = ()> {
    _context: PhantomData<C>,
}

#[kono]
impl<C> Aspect for Introspection<C>
where
    C: 'static,
{
    type Context = C;
    type Environment = Document<'static, String>;

    #[kono::query(rename = "__schema")]
    fn schema(environment: &Document<'static, String>) -> Schema<C> {
        Schema {
            _context: PhantomData,
            schema: environment.to_owned(),
        }
    }
}

pub struct Schema<C = ()> {
    _context: PhantomData<C>,
    schema: Document<'static, String>,
}

impl<C> Schema<C> {
    fn find_type_definition(&self, name: &str) -> Option<&TypeDefinition<'static, String>> {
        self.schema
            .definitions
            .iter()
            .flat_map(|definition| match definition {
                Definition::TypeDefinition(ty) if ty.name() == name => Some(ty),
                _ => None,
            })
            .next()
    }
}

#[kono]
impl<C> Aspect for Schema<C>
where
    C: 'static,
{
    type Context = C;

    #[kono::field(rename = "queryType")]
    fn query_type(&self) -> Type<C> {
        let definition = self.find_type_definition("Query").cloned().unwrap();

        Type {
            _context: PhantomData,
            definition,
        }
    }
}

pub struct Type<C = ()> {
    _context: PhantomData<C>,
    definition: TypeDefinition<'static, String>,
}

#[kono]
impl<C> Aspect for Type<C>
where
    C: 'static,
{
    type Context = C;

    #[kono::field]
    fn name(&self) -> Option<&str> {
        Some(self.definition.name())
    }

    #[kono::field]
    fn description(&self) -> Option<&str> {
        self.definition.description()
    }
}

// fn introspection<C>(schema: Document<'static, String>) -> impl Resolver<Context = C> {}

#[derive(Kono)]
pub enum TypeKind {}
