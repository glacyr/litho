use std::sync::Arc;

use litho_language::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dependency<T> {
    Query,
    Mutation,
    Subscription,
    Schema,
    Type(T),
    Directive(T),
    Fragment(T),
}

pub trait Producer<T> {
    fn product(&self) -> Option<Dependency<T>>;
}

pub trait Consumer<T> {
    fn consumes(&self) -> Vec<Dependency<T>>;
}

impl<T, N> Consumer<T> for N
where
    N: Producer<T> + Node<T>,
    T: ToOwned<Owned = T>,
{
    fn consumes(&self) -> Vec<Dependency<T>> {
        let mut consumes = self.product().into_iter().collect();
        self.traverse(&Tracker, &mut consumes);
        consumes
    }
}

pub struct Tracker;

impl<'a, T> Visit<'a, T> for Tracker
where
    T: ToOwned<Owned = T>,
{
    type Accumulator = Vec<Dependency<T>>;

    fn visit_operation_definition(
        &self,
        node: &'a Arc<OperationDefinition<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        accumulator.push(match node.ty {
            Some(OperationType::Query(_)) | None => Dependency::Query,
            Some(OperationType::Mutation(_)) => Dependency::Mutation,
            Some(OperationType::Subscription(_)) => Dependency::Subscription,
        })
    }

    fn visit_named_type(&self, node: &'a NamedType<T>, accumulator: &mut Self::Accumulator) {
        accumulator.push(Dependency::Type(node.0.as_ref().to_owned()))
    }
}

impl<T> Producer<T> for Definition<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        match self {
            Definition::ExecutableDefinition(definition) => definition.product(),
            Definition::TypeSystemDefinitionOrExtension(definition) => definition.product(),
        }
    }
}

impl<T> Producer<T> for ExecutableDefinition<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        match self {
            ExecutableDefinition::FragmentDefinition(definition) => definition.product(),
            ExecutableDefinition::OperationDefinition(definition) => definition.product(),
        }
    }
}

impl<T> Producer<T> for FragmentDefinition<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        self.fragment_name
            .ok()
            .map(|name| Dependency::Fragment(name.as_ref().to_owned()))
    }
}

impl<T> Producer<T> for OperationDefinition<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        self.name
            .ok()
            .map(|name| Dependency::Fragment(name.as_ref().to_owned()))
    }
}

impl<T> Producer<T> for TypeSystemDefinitionOrExtension<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        match self {
            TypeSystemDefinitionOrExtension::TypeSystemDefinition(definition) => {
                definition.product()
            }
            TypeSystemDefinitionOrExtension::TypeSystemExtension(definition) => {
                definition.product()
            }
        }
    }
}

impl<T> Producer<T> for TypeSystemDefinition<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        match self {
            TypeSystemDefinition::DirectiveDefinition(definition) => definition.product(),
            TypeSystemDefinition::SchemaDefinition(definition) => definition.product(),
            TypeSystemDefinition::TypeDefinition(definition) => definition.product(),
        }
    }
}

impl<T> Producer<T> for DirectiveDefinition<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        self.name
            .ok()
            .map(|name| Dependency::Directive(name.as_ref().to_owned()))
    }
}

impl<T> Producer<T> for SchemaDefinition<T> {
    fn product(&self) -> Option<Dependency<T>> {
        Some(Dependency::Schema)
    }
}

impl<T> Producer<T> for TypeDefinition<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        self.name()
            .ok()
            .map(|name| Dependency::Type(name.as_ref().to_owned()))
    }
}

impl<T> Producer<T> for TypeSystemExtension<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        match self {
            TypeSystemExtension::SchemaExtension(extension) => extension.product(),
            TypeSystemExtension::TypeExtension(extension) => extension.product(),
        }
    }
}

impl<T> Producer<T> for SchemaExtension<T> {
    fn product(&self) -> Option<Dependency<T>> {
        Some(Dependency::Schema)
    }
}

impl<T> Producer<T> for TypeExtension<T>
where
    T: ToOwned<Owned = T>,
{
    fn product(&self) -> Option<Dependency<T>> {
        self.name().map(|name| Dependency::Type(name.to_owned()))
    }
}
