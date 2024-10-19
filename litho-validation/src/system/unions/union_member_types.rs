use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct UnionMemberTypes<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> UnionMemberTypes<'a, T>
where
    T: Eq + Hash + ToString,
{
    fn check_union_member_types(
        &self,
        name: &T,
        member_types: &litho_language::ast::UnionMemberTypes<T>,
    ) -> Vec<Diagnostic<Span>> {
        let mut errors = vec![];

        for ty in member_types.named_types() {
            match self
                .0
                .union_member_types_by_name(name, ty.0.as_ref())
                .next()
            {
                Some(first) if !Arc::ptr_eq(first, ty) => {
                    errors.push(Diagnostic::duplicate_union_member(
                        ty.0.as_ref().to_string(),
                        first.span(),
                        ty.span(),
                    ));
                    continue;
                }
                Some(_) | None => {}
            }

            if !self.0.is_object_type(ty.0.as_ref()) {
                errors.push(Diagnostic::non_object_union_member(
                    ty.0.as_ref().to_string(),
                    ty.span(),
                ));
            }
        }

        errors
    }
}

impl<'a, T> Visit<'a, T> for UnionMemberTypes<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_union_type_definition(
        &self,
        node: &'a UnionTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        if let Some(member_types) = node.member_types.as_ref() {
            accumulator.extend(self.check_union_member_types(name.as_ref(), member_types));
        }

        if self.0.union_member_types(name.as_ref()).next().is_none() {
            accumulator.push(Diagnostic::missing_union_members(
                name.as_ref().to_string(),
                name.span(),
            ));
        }
    }

    fn visit_union_type_extension(
        &self,
        node: &'a UnionTypeExtension<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        if let Some(member_types) = node.member_types.as_ref() {
            accumulator.extend(self.check_union_member_types(name.0.as_ref(), member_types));
        }
    }
}
