use std::borrow::Borrow;
use std::hash::Hash;
use std::sync::Arc;

use litho_language::ast::*;
use multimap::MultiMap;

#[derive(Debug)]
pub struct Database<T>
where
    T: Eq + Hash,
{
    operation_definitions_by_name: MultiMap<T, Arc<OperationDefinition<T>>>,
    fragment_definitions_by_name: MultiMap<T, Arc<FragmentDefinition<T>>>,
    type_definitions_by_name: MultiMap<T, Arc<TypeDefinition<T>>>,
    type_extensions_by_name: MultiMap<T, Arc<TypeExtension<T>>>,
    field_definitions: MultiMap<T, Arc<FieldDefinition<T>>>,
}

impl<T> Default for Database<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Database {
            operation_definitions_by_name: Default::default(),
            fragment_definitions_by_name: Default::default(),
            type_definitions_by_name: Default::default(),
            type_extensions_by_name: Default::default(),
            field_definitions: Default::default(),
        }
    }
}

impl<T> Database<T>
where
    T: Clone + std::fmt::Debug + Eq + Hash,
{
    pub fn new(document: &Document<T>) -> Database<T> {
        eprintln!("Building database.");

        let mut database = Default::default();
        document.traverse(&Index, &mut database);
        database
    }

    pub fn type_definitions_by_name(&self, name: &T) -> impl Iterator<Item = &TypeDefinition<T>> {
        self.type_definitions_by_name
            .get_vec(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(AsRef::as_ref)
    }
}

pub struct Index;

impl<'ast, T> Visit<'ast, T> for Index
where
    T: Clone + std::fmt::Debug + Eq + Hash + 'ast,
{
    type Accumulator = Database<T>;

    fn visit_type_definition(
        &self,
        node: &'ast Arc<TypeDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        eprintln!("Visiting operation definition.");

        if let Some(name) = node.name().ok() {
            eprintln!("Got a name: {:#?}", name.as_ref());
            accumulator
                .type_definitions_by_name
                .insert(name.as_ref().clone(), node.clone());
        }
    }

    // fn visit_fragment_definition(
    //     &self,
    //     node: &'ast FragmentDefinition<T>,
    //     accumulator: &mut Self::Accumulator,
    // ) {
    //     if let Some(name) = node.fragment_name.ok() {
    //         accumulator
    //             .fragment_definitions_by_name
    //             .insert(name.as_ref().clone(), node.clone());
    //     }
    // }

    // fn visit_type_definition(
    //     &self,
    //     node: &'ast TypeDefinition<T>,
    //     accumulator: &mut Self::Accumulator,
    // ) {
    //     if let Some(name) = node.name().ok() {
    //         accumulator
    //             .type_definitions_by_name
    //             .insert(name.as_ref().clone(), node.clone());
    //     }
    // }

    // fn visit_type_extension(
    //     &self,
    //     node: &'ast TypeExtension<T>,
    //     accumulator: &mut Self::Accumulator,
    // ) {
    //     if let Some(name) = node.name().ok() {
    //         accumulator
    //             .type_extensions_by_name
    //             .insert(name.as_ref().clone(), node.clone());
    //     }
    // }

    // fn visit_object_type_definition(
    //     &self,
    //     node: &'ast ObjectTypeDefinition<T>,
    //     accumulator: &mut Self::Accumulator,
    // ) {
    //     if let Some(name) = node.name.ok() {
    //         for field in node
    //             .fields_definition
    //             .iter()
    //             .flat_map(|definition| definition.definitions.iter())
    //         {
    //             accumulator.field_definitions.insert(name.as_ref(), field);
    //         }
    //     }
    // }
}
