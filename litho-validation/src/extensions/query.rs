use graphql_parser::query::{
    Definition, Document, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    OperationDefinition, Query, Selection, SelectionSet, Text, VariableDefinition,
};
use graphql_parser::schema::Directive;
use graphql_parser::Pos;

use super::Span;

pub trait DefinitionExt<'a, T>
where
    T: Text<'a>,
{
    fn ty(&self) -> T::Value;
    fn name(&self) -> Option<&T::Value>;
    fn selection_set(&self) -> &SelectionSet<'a, T>;
    fn position(&self) -> Pos;
    fn span(&self) -> Span;
}

impl<'a, T> DefinitionExt<'a, T> for Definition<'a, T>
where
    T: Text<'a>,
{
    fn ty(&self) -> T::Value {
        match self {
            Definition::Fragment(fragment) => fragment.name.to_owned(),
            Definition::Operation(operation) => match operation {
                OperationDefinition::Mutation(_) => "Mutation".into(),
                OperationDefinition::Query(_) | OperationDefinition::SelectionSet(_) => {
                    "Query".into()
                }
                OperationDefinition::Subscription(_) => "Subscription".into(),
            },
        }
    }

    fn name(&self) -> Option<&T::Value> {
        match self {
            Definition::Fragment(fragment) => Some(&fragment.name),
            Definition::Operation(operation) => match operation {
                OperationDefinition::Mutation(mutation) => mutation.name.as_ref(),
                OperationDefinition::Query(query) => query.name.as_ref(),
                OperationDefinition::SelectionSet(_) => None,
                OperationDefinition::Subscription(sub) => sub.name.as_ref(),
            },
        }
    }

    fn selection_set(&self) -> &SelectionSet<'a, T> {
        match self {
            Definition::Fragment(fragment) => &fragment.selection_set,
            Definition::Operation(operation) => match operation {
                OperationDefinition::Mutation(mutation) => &mutation.selection_set,
                OperationDefinition::Query(query) => &query.selection_set,
                OperationDefinition::SelectionSet(set) => &set,
                OperationDefinition::Subscription(sub) => &sub.selection_set,
            },
        }
    }

    fn position(&self) -> Pos {
        match self {
            Definition::Fragment(fragment) => fragment.position(),
            Definition::Operation(operation) => operation.position(),
        }
    }

    fn span(&self) -> Span {
        match self {
            Definition::Fragment(fragment) => fragment.span(),
            Definition::Operation(operation) => operation.span(),
        }
    }
}

pub trait DirectiveExt<'a, T>
where
    T: Text<'a>,
{
    fn position(&self) -> Pos;
    fn span(&self) -> Span;
}

impl<'a, T> DirectiveExt<'a, T> for Directive<'a, T>
where
    T: Text<'a>,
{
    fn position(&self) -> Pos {
        self.position
    }

    fn span(&self) -> Span {
        Span(self.position, self.name.as_ref().len() + 1)
    }
}

pub trait FragmentDefinitionExt<'a, T>
where
    T: Text<'a>,
{
    fn position(&self) -> Pos;
    fn span(&self) -> Span;
}

impl<'a, T> FragmentDefinitionExt<'a, T> for FragmentDefinition<'a, T>
where
    T: Text<'a>,
{
    fn position(&self) -> Pos {
        self.position
    }

    fn span(&self) -> Span {
        Span(self.position, "fragment".len())
    }
}

pub trait FragmentSpreadExt<'a, T>
where
    T: Text<'a>,
{
    fn position(&self) -> Pos;
    fn span(&self) -> Span;
}

impl<'a, T> FragmentSpreadExt<'a, T> for FragmentSpread<'a, T>
where
    T: Text<'a>,
{
    fn position(&self) -> Pos {
        self.position
    }

    fn span(&self) -> Span {
        Span(self.position, self.fragment_name.as_ref().len())
    }
}

pub trait OperationDefinitionExt<'a, T>
where
    T: Text<'a>,
{
    fn name(&self) -> Option<&T::Value>;
    fn variable_definitions(&self) -> &[VariableDefinition<'a, T>];
    fn selection_set(&self) -> &SelectionSet<'a, T>;
    fn position(&self) -> Pos;
    fn span(&self) -> Span;
}

impl<'a, T> OperationDefinitionExt<'a, T> for OperationDefinition<'a, T>
where
    T: Text<'a>,
{
    fn name(&self) -> Option<&T::Value> {
        match self {
            OperationDefinition::Mutation(mutation) => mutation.name.as_ref(),
            OperationDefinition::Query(query) => query.name.as_ref(),
            OperationDefinition::SelectionSet(_) => None,
            OperationDefinition::Subscription(sub) => sub.name.as_ref(),
        }
    }

    fn variable_definitions(&self) -> &[VariableDefinition<'a, T>] {
        match self {
            OperationDefinition::Mutation(mutation) => &mutation.variable_definitions,
            OperationDefinition::Query(query) => &query.variable_definitions,
            OperationDefinition::SelectionSet(_) => &[],
            OperationDefinition::Subscription(sub) => &sub.variable_definitions,
        }
    }

    fn selection_set(&self) -> &SelectionSet<'a, T> {
        match self {
            OperationDefinition::Mutation(mutation) => &mutation.selection_set,
            OperationDefinition::Query(query) => &query.selection_set,
            OperationDefinition::SelectionSet(set) => set,
            OperationDefinition::Subscription(sub) => &sub.selection_set,
        }
    }

    fn position(&self) -> Pos {
        match self {
            OperationDefinition::Mutation(mutation) => mutation.position,
            OperationDefinition::Query(query) => query.position,
            OperationDefinition::SelectionSet(set) => set.span.0,
            OperationDefinition::Subscription(sub) => sub.position,
        }
    }

    fn span(&self) -> Span {
        let len = match self {
            OperationDefinition::Mutation(_) => "mutation".len(),
            OperationDefinition::Query(_) => "query".len(),
            OperationDefinition::SelectionSet(_) => 0,
            OperationDefinition::Subscription(_) => "subscription".len(),
        };

        Span(self.position(), len)
    }
}

pub trait FieldExt<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span;
}

impl<'a, T> FieldExt<'a, T> for Field<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span {
        Span(self.position, self.name.as_ref().len())
    }
}

pub trait InlineFragmentExt<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span;
}

impl<'a, T> InlineFragmentExt<'a, T> for InlineFragment<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span {
        Span(self.position, 2)
    }
}

pub trait SelectionExt<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span;
}

impl<'a, T> SelectionExt<'a, T> for Selection<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span {
        match self {
            Selection::Field(field) => field.span(),
            Selection::FragmentSpread(fragment) => Span(fragment.position, 3),
            Selection::InlineFragment(fragment) => fragment.span(),
        }
    }
}

pub trait SelectionSetExt<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span;
}

impl<'a, T> SelectionSetExt<'a, T> for SelectionSet<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span {
        Span(self.span.0, 1)
    }
}

pub trait QueryDocumentExt<'a, T>
where
    T: Text<'a>,
{
    fn fragment_definition(&self, name: &str) -> Option<&FragmentDefinition<'a, T>>;
}

impl<'a, T> QueryDocumentExt<'a, T> for Document<'a, T>
where
    T: Text<'a>,
{
    fn fragment_definition(&self, name: &str) -> Option<&FragmentDefinition<'a, T>> {
        self.definitions
            .iter()
            .find_map(|definition| match definition {
                Definition::Fragment(fragment) if fragment.name.as_ref() == name => Some(fragment),
                Definition::Fragment(_) | Definition::Operation(_) => None,
            })
    }
}

pub trait QueryExt<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span;
}

impl<'a, T> QueryExt<'a, T> for Query<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span {
        Span(self.position, "query".len())
    }
}

pub trait VariableDefinitionExt {
    fn span(&self) -> Span;
}

impl<'a, T> VariableDefinitionExt for VariableDefinition<'a, T>
where
    T: Text<'a>,
{
    fn span(&self) -> Span {
        Span(self.position, 1 + self.name.as_ref().len())
    }
}
