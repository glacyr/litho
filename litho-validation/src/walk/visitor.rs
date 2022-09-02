use graphql_parser::query::{
    Definition, Directive, Document, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    Mutation, OperationDefinition, Query, Selection, SelectionSet, Subscription, Text,
    TypeCondition, Value, VariableDefinition,
};
use graphql_parser::schema::{Document as Schema, Type};

use super::Scope;

#[allow(unused)]
pub trait Visitor<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    type Accumulator;

    fn visit_document(
        &self,
        document: &'v Document<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_definition(
        &self,
        definition: &'v Definition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_fragment_definition(
        &self,
        fragment_definition: &'v FragmentDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_type_condition(
        &self,
        type_condition: &'v TypeCondition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_directive(
        &self,
        directive: &'v Directive<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_value(
        &self,
        value: &'v Value<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_selection_set(
        &self,
        selection_set: &'v SelectionSet<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_selection(
        &self,
        selection: &'v Selection<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_field(
        &self,
        field: &'v Field<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v FragmentSpread<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_inline_fragment(
        &self,
        inline_fragment: &'v InlineFragment<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_operation_definition(
        &self,
        operation_definition: &'v OperationDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_mutation(
        &self,
        mutation: &'v Mutation<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_variable_definition(
        &self,
        variable_definition: &'v VariableDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_type(
        &self,
        ty: &'v Type<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_query(
        &self,
        query: &'v Query<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_subscription(
        &self,
        subscription: &'v Subscription<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope,
        accumulator: &mut Self::Accumulator,
    ) {
    }
}
