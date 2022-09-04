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
        schema: &'v Schema<'a, _T>,
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
        impl<'v, 'a, _T, $first, $($second),*> Visitor<'v, 'a, _T> for ($first, $($second),*)
        where
            'a: 'v,
            $first: Visitor<'v, 'a, _T>,
            $($second: Visitor<'v, 'a, _T, Accumulator = $first::Accumulator>,)*
            _T: Text<'a>,
        {
            type Accumulator = $first::Accumulator;

            branch!(($first $($second)*), visit_document, document: &'v Document<'a, _T>);
            branch!(($first $($second)*), visit_definition, definition: &'v Definition<'a, _T>);
            branch!(($first $($second)*), visit_fragment_definition, fragment_definition: &'v FragmentDefinition<'a, _T>);
            branch!(($first $($second)*), visit_type_condition, type_condition: &'v TypeCondition<'a, _T>);
            branch!(($first $($second)*), visit_directive, directive: &'v Directive<'a, _T>);
            branch!(($first $($second)*), visit_value, value: &'v Value<'a, _T>);
            branch!(($first $($second)*), visit_selection_set, selection_set: &'v SelectionSet<'a, _T>);
            branch!(($first $($second)*), visit_selection, selection: &'v Selection<'a, _T>);
            branch!(($first $($second)*), visit_field, field: &'v Field<'a, _T>);
            branch!(($first $($second)*), visit_fragment_spread, fragment_spread: &'v FragmentSpread<'a, _T>);
            branch!(($first $($second)*), visit_inline_fragment, inline_fragment: &'v InlineFragment<'a, _T>);
            branch!(($first $($second)*), visit_operation_definition, operation_definition: &'v OperationDefinition<'a, _T>);
            branch!(($first $($second)*), visit_mutation, mutation: &'v Mutation<'a, _T>);
            branch!(($first $($second)*), visit_variable_definition, variable_definition: &'v VariableDefinition<'a, _T>);
            branch!(($first $($second)*), visit_type, ty: &'v Type<'a, _T>);
            branch!(($first $($second)*), visit_query, ty: &'v Query<'a, _T>);
            branch!(($first $($second)*), visit_subscription, ty: &'v Subscription<'a, _T>);
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
impl_tuple!(A B C D E F G H I J K L M);
impl_tuple!(A B C D E F G H I J K L M N);
impl_tuple!(A B C D E F G H I J K L M N O);
impl_tuple!(A B C D E F G H I J K L M N O P);
impl_tuple!(A B C D E F G H I J K L M N O P Q);
impl_tuple!(A B C D E F G H I J K L M N O P Q R);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U V);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U V W);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U V W X);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U V W X Y);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z A0);
