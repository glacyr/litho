use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct FieldSelectionMerging<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> FieldSelectionMerging<'a, T>
where
    T: Eq + Hash + ToString,
{
    fn fields_by_name(
        &self,
        set: &'a Arc<SelectionSet<T>>,
        fields: &mut HashMap<&'a T, Vec<(&'a T, &'a Arc<Field<T>>)>>,
    ) {
        let ty = match self.0.inference.type_by_selection_set.get(set) {
            Some(ty) => ty,
            None => return,
        };

        for selection in set.selections.iter() {
            match selection {
                Selection::Field(field) => {
                    let name = field
                        .alias
                        .as_ref()
                        .map(|alias| &alias.name)
                        .or(field.name.ok())
                        .map(AsRef::as_ref);

                    if let Some(name) = name {
                        fields.entry(name).or_default().push((ty, field));
                    }
                }
                Selection::FragmentSpread(fragment) => {
                    let selection_set = self
                        .0
                        .operations
                        .by_name(fragment.fragment_name.as_ref())
                        .next()
                        .and_then(|definition| definition.selection_set.ok());

                    if let Some(selection_set) = selection_set {
                        self.fields_by_name(selection_set, fields);
                    }
                }
                Selection::InlineFragment(fragment) => {
                    if let Some(selection_set) = fragment.selection_set.ok() {
                        self.fields_by_name(selection_set, fields);
                    }
                }
            }
        }
    }

    fn pairs<'b, U>(&self, fields: &'b [U]) -> impl Iterator<Item = (&'b U, &'b U)>
    where
        U: 'a,
    {
        (0..fields.len())
            .flat_map(|i| (i + 1..fields.len()).map(move |j| (i, j)))
            .map(|(i, j)| (&fields[i], &fields[j]))
    }

    fn fields_can_merge(
        &self,
        response_key: &T,
        fields: &[(&'a T, &'a Arc<Field<T>>)],
    ) -> Vec<Diagnostic<Span>> {
        let mut diagnostics = vec![];

        for (a, b) in self.pairs(fields) {
            diagnostics.extend(self.check_pair(response_key, a, b).into_iter());
        }

        diagnostics
    }

    fn check_pair(
        &self,
        response_key: &T,
        a: &(&T, &Arc<Field<T>>),
        b: &(&T, &Arc<Field<T>>),
    ) -> Option<Diagnostic<Span>> {
        let span_a =
            a.1.alias
                .as_ref()
                .map(|alias| alias.name.span())
                .unwrap_or(a.1.name.span());
        let span_b =
            b.1.alias
                .as_ref()
                .map(|alias| alias.name.span())
                .unwrap_or(b.1.name.span());

        if self.same_response_shape(a.1, b.1)? == false {
            return Some(Diagnostic::incompatible_response_shape(
                response_key.to_string(),
                span_a,
                span_b,
            ));
        }

        if a.0 == b.0 || !self.0.is_object_type(&a.0) || !self.0.is_object_type(&b.0) {
            if a.1.name.ok()?.as_ref() != b.1.name.ok()?.as_ref() {
                return Some(Diagnostic::different_field_names(
                    response_key.to_string(),
                    span_a,
                    span_b,
                ));
            }

            if !a.1.arguments.as_ref().congruent(&b.1.arguments.as_ref()) {
                return Some(Diagnostic::different_field_arguments(
                    response_key.to_string(),
                    span_a,
                    span_b,
                ));
            }

            let mut fields_by_name = HashMap::new();

            if let Some(set) = a.1.selection_set.as_ref() {
                self.fields_by_name(set, &mut fields_by_name);
            }

            if let Some(set) = b.1.selection_set.as_ref() {
                self.fields_by_name(set, &mut fields_by_name);
            }

            for (response_key, fields) in fields_by_name.iter() {
                if !self.fields_can_merge(response_key, fields).is_empty() {
                    return Some(Diagnostic::incompatible_response_fields(
                        response_key.to_string(),
                        span_a,
                        span_b,
                    ));
                }
            }
        }

        None
    }

    fn same_response_shape(&self, a: &Arc<Field<T>>, b: &Arc<Field<T>>) -> Option<bool> {
        let fields = &self.0.inference.field_definitions_by_field;

        let ty_a = fields.get(a)?.ty.ok()?;
        let ty_b = fields.get(b)?.ty.ok()?;

        if !self.same_response_shape_type(ty_a, ty_b)? {
            return Some(false);
        }

        let mut fields_by_name = HashMap::new();

        if let Some(set) = a.selection_set.as_ref() {
            self.fields_by_name(set, &mut fields_by_name);
        }

        if let Some(set) = b.selection_set.as_ref() {
            self.fields_by_name(set, &mut fields_by_name);
        }

        for (_, fields) in fields_by_name.iter() {
            let pairs = self.pairs(fields);

            for (first, second) in pairs {
                if !self.same_response_shape(first.1, second.1)? {
                    return Some(false);
                }
            }
        }

        Some(true)
    }

    fn same_response_shape_type(&self, ty_a: &Arc<Type<T>>, ty_b: &Arc<Type<T>>) -> Option<bool> {
        let (ty_a, ty_b) = match (ty_a.as_ref(), ty_b.as_ref()) {
            (Type::NonNull(ty_a), Type::NonNull(ty_b)) => (ty_a.ty.as_ref(), ty_b.ty.as_ref()),
            (Type::NonNull(_), _) | (_, Type::NonNull(_)) => return Some(false),
            (ty_a, ty_b) => (ty_a, ty_b),
        };

        match (ty_a, ty_b) {
            (Type::List(ty_a), Type::List(ty_b)) => {
                return self.same_response_shape_type(ty_a.ty.ok()?, ty_b.ty.ok()?)
            }
            (Type::List(_), _) | (_, Type::List(_)) => return Some(false),
            (_, _) => {}
        };

        let def_a = self.0.type_definitions_by_name(ty_a.name()?).next()?;
        let def_b = self.0.type_definitions_by_name(ty_b.name()?).next()?;

        if def_a.is_scalar_like() || def_b.is_scalar_like() {
            return Some(ty_a.name()? == ty_b.name()?);
        }

        Some(true)
    }
}

impl<'a, T> Visit<'a, T> for FieldSelectionMerging<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_selection_set(
        &self,
        node: &'a Arc<SelectionSet<T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let mut fields_by_name = HashMap::new();
        self.fields_by_name(node, &mut fields_by_name);

        for (response_key, fields) in fields_by_name.iter() {
            accumulator.extend(self.fields_can_merge(response_key, fields).into_iter());
        }
    }
}
