use litho_language::ast::*;
use multimap::MultiMap;

pub struct Database<'ast, 'a> {
    operation_definitions_by_name: MultiMap<&'ast str, &'ast OperationDefinition<'a>>,
    fragment_definitions_by_name: MultiMap<&'ast str, &'ast FragmentDefinition<'a>>,
    type_definitions_by_name: MultiMap<&'ast str, &'ast TypeDefinition<'a>>,
    type_extensions_by_name: MultiMap<&'ast str, &'ast TypeExtension<'a>>,
    field_definitions: MultiMap<&'ast str, &'ast FieldDefinition<'a>>,
}

pub struct Index;

impl<'ast, 'a> Visit<'ast, 'a> for Index
where
    'a: 'ast,
{
    type Accumulator = Database<'ast, 'a>;

    fn visit_operation_definition(
        &self,
        node: &'ast OperationDefinition<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            accumulator
                .operation_definitions_by_name
                .insert(name.as_ref(), node);
        }
    }

    fn visit_fragment_definition(
        &self,
        node: &'ast FragmentDefinition<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.fragment_name.ok() {
            accumulator
                .fragment_definitions_by_name
                .insert(name.as_ref(), node);
        }
    }

    fn visit_type_definition(
        &self,
        node: &'ast TypeDefinition<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name().ok() {
            accumulator
                .type_definitions_by_name
                .insert(name.as_ref(), node);
        }
    }

    fn visit_type_extension(
        &self,
        node: &'ast TypeExtension<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name().ok() {
            accumulator
                .type_extensions_by_name
                .insert(name.as_ref(), node);
        }
    }

    fn visit_object_type_definition(
        &self,
        node: &'ast ObjectTypeDefinition<'a>,
        accumulator: &mut Self::Accumulator,
    ) {
        if let Some(name) = node.name.ok() {
            for field in node
                .fields_definition
                .iter()
                .flat_map(|definition| definition.definitions.iter())
            {
                accumulator.field_definitions.insert(name.as_ref(), field);
            }
        }
    }
}

pub trait Query<'ast, 'a> {
    fn type_definitions_by_name<'b>(&'b self, name: &'b str) -> &'b [&'ast TypeDefinition<'a>];
}

impl<'ast, 'a> Query<'ast, 'a> for Database<'ast, 'a> {
    fn type_definitions_by_name<'b>(&'b self, name: &'b str) -> &'b [&'ast TypeDefinition<'a>] {
        self.type_definitions_by_name
            .get_vec(&name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}
