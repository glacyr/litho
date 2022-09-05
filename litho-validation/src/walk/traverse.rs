use graphql_parser::query::{
    Definition, Directive, Document, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    Mutation, OperationDefinition, Query, Selection, SelectionSet, Subscription, Text,
    TypeCondition, Value, VariableDefinition,
};
use graphql_parser::schema::{Document as Schema, Type};

use crate::extensions::*;

use super::{Scope, Visitor};

pub trait Traverse<'v, 'a, T>: Visitor<'v, 'a, T>
where
    'a: 'v,
    T: Text<'a>,
{
    fn traverse(
        &self,
        document: &'v Document<'a, T>,
        schema: &'v Schema<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.traverse_document(document, schema, &Scope::Document, accumulator);
    }

    fn traverse_document(
        &self,
        document: &'v Document<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_document(document, schema, scope, accumulator);

        for definition in document.definitions.iter() {
            self.traverse_definition(definition, schema, scope, accumulator);
        }
    }

    fn traverse_definition(
        &self,
        definition: &'v Definition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_definition(definition, schema, scope, accumulator);

        match definition {
            Definition::Fragment(fragment) => {
                self.traverse_fragment_definition(fragment, schema, scope, accumulator)
            }
            Definition::Operation(operation) => {
                self.traverse_operation_definition(operation, schema, scope, accumulator)
            }
        }
    }

    fn traverse_fragment_definition(
        &self,
        fragment_definition: &'v FragmentDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_fragment_definition(fragment_definition, schema, scope, accumulator);

        self.traverse_type_condition(
            &fragment_definition.type_condition,
            schema,
            scope,
            accumulator,
        );

        for directive in fragment_definition.directives.iter() {
            self.traverse_directive(directive, schema, scope, accumulator);
        }

        let ty = match fragment_definition.type_condition {
            TypeCondition::On(ref ty) => ty,
        };

        let scope = scope.fragment(
            fragment_definition.name.as_ref(),
            ty.as_ref(),
            fragment_definition.span(),
        );

        self.traverse_selection_set(
            &fragment_definition.selection_set,
            schema,
            &scope,
            accumulator,
        );
    }

    fn traverse_type_condition(
        &self,
        type_condition: &'v TypeCondition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_type_condition(type_condition, schema, scope, accumulator);
    }

    fn traverse_directive(
        &self,
        directive: &'v Directive<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_directive(directive, schema, scope, accumulator);

        for (_, value) in directive.arguments.iter() {
            self.traverse_value(value, schema, scope, accumulator);
        }
    }

    fn traverse_value(
        &self,
        value: &'v Value<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_value(value, schema, scope, accumulator);
    }

    fn traverse_selection_set(
        &self,
        selection_set: &'v SelectionSet<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_selection_set(selection_set, schema, scope, accumulator);

        for item in selection_set.items.iter() {
            self.traverse_selection(item, schema, scope, accumulator);
        }
    }

    fn traverse_selection(
        &self,
        selection: &'v Selection<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_selection(selection, schema, scope, accumulator);

        match selection {
            Selection::Field(field) => self.traverse_field(field, schema, scope, accumulator),
            Selection::FragmentSpread(spread) => {
                self.traverse_fragment_spread(spread, schema, scope, accumulator)
            }
            Selection::InlineFragment(fragment) => {
                self.traverse_inline_fragment(fragment, schema, scope, accumulator)
            }
        }
    }

    fn traverse_field(
        &self,
        field: &'v Field<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_field(field, schema, scope, accumulator);

        for directive in field.directives.iter() {
            self.traverse_directive(directive, schema, scope, accumulator);
        }

        let ty = match schema.type_definition(scope.ty()) {
            Some(ty) => ty,
            None => return,
        };

        let ty = match ty.field(&field.name) {
            Some(field) => field.field_type.name().as_ref(),
            None => return,
        };

        let scope = scope.field(field.name.as_ref(), field.span(), ty);

        for (_, value) in field.arguments.iter() {
            self.traverse_value(value, schema, &scope, accumulator);
        }

        self.traverse_selection_set(&field.selection_set, schema, &scope, accumulator);
    }

    fn traverse_fragment_spread(
        &self,
        fragment_spread: &'v FragmentSpread<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_fragment_spread(fragment_spread, schema, scope, accumulator);

        for directive in fragment_spread.directives.iter() {
            self.traverse_directive(directive, schema, scope, accumulator);
        }
    }

    fn traverse_inline_fragment(
        &self,
        inline_fragment: &'v InlineFragment<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_inline_fragment(inline_fragment, schema, scope, accumulator);

        if let Some(ref type_condition) = inline_fragment.type_condition {
            self.traverse_type_condition(type_condition, schema, scope, accumulator);
        }

        for directive in inline_fragment.directives.iter() {
            self.traverse_directive(directive, schema, scope, accumulator);
        }

        let scope = &match inline_fragment.type_condition {
            Some(TypeCondition::On(ref ty)) => {
                scope.inline_fragment(ty.as_ref(), inline_fragment.span())
            }
            None => *scope,
        };

        self.traverse_selection_set(&inline_fragment.selection_set, schema, scope, accumulator);
    }

    fn traverse_operation_definition(
        &self,
        operation_definition: &'v OperationDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_operation_definition(operation_definition, schema, scope, accumulator);

        match operation_definition {
            OperationDefinition::Mutation(mutation) => self.traverse_mutation(
                mutation,
                schema,
                &scope.mutation(operation_definition.span()),
                accumulator,
            ),
            OperationDefinition::Query(query) => self.traverse_query(
                query,
                schema,
                &scope.query(operation_definition.span()),
                accumulator,
            ),
            OperationDefinition::SelectionSet(selection_set) => {
                self.traverse_selection_set(selection_set, schema, scope, accumulator)
            }
            OperationDefinition::Subscription(subscription) => self.traverse_subscription(
                subscription,
                schema,
                &scope.subscription(operation_definition.span()),
                accumulator,
            ),
        }
    }

    fn traverse_mutation(
        &self,
        mutation: &'v Mutation<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_mutation(mutation, schema, scope, accumulator);

        for variable_definition in mutation.variable_definitions.iter() {
            self.traverse_variable_definition(variable_definition, schema, scope, accumulator);
        }

        for directive in mutation.directives.iter() {
            self.traverse_directive(directive, schema, scope, accumulator);
        }

        self.traverse_selection_set(&mutation.selection_set, schema, scope, accumulator);
    }

    fn traverse_variable_definition(
        &self,
        variable_definition: &'v VariableDefinition<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_variable_definition(variable_definition, schema, scope, accumulator);

        self.traverse_type(&variable_definition.var_type, schema, scope, accumulator);

        if let Some(ref default_value) = variable_definition.default_value {
            self.traverse_value(default_value, schema, scope, accumulator);
        }
    }

    fn traverse_type(
        &self,
        ty: &'v Type<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_type(ty, schema, scope, accumulator);
    }

    fn traverse_query(
        &self,
        query: &'v Query<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_query(query, schema, scope, accumulator);

        for variable_definition in query.variable_definitions.iter() {
            self.traverse_variable_definition(variable_definition, schema, scope, accumulator);
        }

        for directive in query.directives.iter() {
            self.traverse_directive(directive, schema, scope, accumulator);
        }

        self.traverse_selection_set(&query.selection_set, schema, scope, accumulator);
    }

    fn traverse_subscription(
        &self,
        subscription: &'v Subscription<'a, T>,
        schema: &'v Schema<'a, T>,
        scope: &Scope<'_, 'v>,
        accumulator: &mut Self::Accumulator,
    ) {
        self.visit_subscription(subscription, schema, scope, accumulator);

        for variable_definition in subscription.variable_definitions.iter() {
            self.traverse_variable_definition(variable_definition, schema, scope, accumulator);
        }

        for directive in subscription.directives.iter() {
            self.traverse_directive(directive, schema, scope, accumulator);
        }

        self.traverse_selection_set(&subscription.selection_set, schema, scope, accumulator);
    }
}

macro_rules! impl_tuple (
    ($first:ident $($second:ident)*) => {
        #[allow(non_snake_case)]
        impl<'v, 'a, _T, $first, $($second),*> Traverse<'v, 'a, _T> for ($first, $($second),*)
        where
            'a: 'v,
            $first: Traverse<'v, 'a, _T>,
            $($second: Traverse<'v, 'a, _T, Accumulator = $first::Accumulator>,)*
            _T: Text<'a>,
        {
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
impl_tuple!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z A0 A1);
