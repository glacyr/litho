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
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_definition(
        &self,
        definition: &'v Definition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_fragment_definition(
        &self,
        fragment_definition: &'v FragmentDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_type_condition(
        &self,
        type_condition: &'v TypeCondition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_directive(
        &self,
        directive: &'v Directive<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_value(
        &self,
        value: &'v Value<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
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
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_field(
        &self,
        field: &'v Field<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_fragment_spread(
        &self,
        fragment_spread: &'v FragmentSpread<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_inline_fragment(
        &self,
        inline_fragment: &'v InlineFragment<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_operation_definition(
        &self,
        operation_definition: &'v OperationDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_mutation(
        &self,
        mutation: &'v Mutation<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_variable_definition(
        &self,
        variable_definition: &'v VariableDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_type(
        &self,
        ty: &'v Type<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_query(
        &self,
        query: &'v Query<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }

    fn visit_subscription(
        &self,
        subscription: &'v Subscription<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
    }
}

macro_rules! branch {
    (($($gen:ident)*), $name:ident, $arg:ident : $ty:ty) => {
    fn $name(
        &self,
        $arg: $ty,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        let ($(ref $gen,)*) = self;

        $(
            $gen.$name($arg, schema, scope, accumulator);
        )*
    }
    };
}

macro_rules! impl_tuple (
    ($first:ident $($second:ident)*) => {
        #[allow(non_snake_case)]
        impl<'v, 'a, T, $first, $($second),*> Visitor<'v, 'a, T> for ($first, $($second),*)
        where
            'a: 'v,
            $first: Visitor<'v, 'a, T>,
            $($second: Visitor<'v, 'a, T, Accumulator = $first::Accumulator>,)*
            T: Text<'a>,
        {
            type Accumulator = $first::Accumulator;

            branch!(($first $($second)*), visit_document, document: &'v Document<'a, T>);
            branch!(($first $($second)*), visit_definition, definition: &'v Definition<'a, T>);
            branch!(($first $($second)*), visit_fragment_definition, fragment_definition: &'v FragmentDefinition<'a, T>);
            branch!(($first $($second)*), visit_type_condition, type_condition: &'v TypeCondition<'a, T>);
            branch!(($first $($second)*), visit_directive, directive: &'v Directive<'a, T>);
            branch!(($first $($second)*), visit_value, value: &'v Value<'a, T>);
            branch!(($first $($second)*), visit_selection_set, selection_set: &'v SelectionSet<'a, T>);
            branch!(($first $($second)*), visit_selection, selection: &'v Selection<'a, T>);
            branch!(($first $($second)*), visit_field, field: &'v Field<'a, T>);
            branch!(($first $($second)*), visit_fragment_spread, fragment_spread: &'v FragmentSpread<'a, T>);
            branch!(($first $($second)*), visit_inline_fragment, inline_fragment: &'v InlineFragment<'a, T>);
            branch!(($first $($second)*), visit_operation_definition, operation_definition: &'v OperationDefinition<'a, T>);
            branch!(($first $($second)*), visit_mutation, mutation: &'v Mutation<'a, T>);
            branch!(($first $($second)*), visit_variable_definition, variable_definition: &'v VariableDefinition<'a, T>);
            branch!(($first $($second)*), visit_type, ty: &'v Type<'a, T>);
            branch!(($first $($second)*), visit_query, ty: &'v Query<'a, T>);
            branch!(($first $($second)*), visit_subscription, ty: &'v Subscription<'a, T>);
        }
    }
);

impl_tuple!(A);
impl_tuple!(A B);
impl_tuple!(A B C);
impl_tuple!(A B C D);
impl_tuple!(A B C D E);
impl_tuple!(A B C D E F);
impl_tuple!(A B C D E F G);
impl_tuple!(A B C D E F G H);
impl_tuple!(A B C D E F G H I);
impl_tuple!(A B C D E F G H I J);
impl_tuple!(A B C D E F G H I J K);
impl_tuple!(A B C D E F G H I J K L);
