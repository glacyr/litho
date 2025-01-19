use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct UnionMemberTypes<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> UnionMemberTypes<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    fn check_union_member_types(
        &self,
        name: &T,
        member_types: &litho_language::ast::UnionMemberTypes<'a, T>,
    ) -> Vec<Diagnostic<Span>> {
        let mut errors = vec![];

        for ty in member_types.named_types() {
            match self
                .0
                .union_member_types_by_name(name, ty.0.as_ref())
                .next()
            {
                Some(first) if !Shared::ptr_eq(first, ty) => {
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

impl<'ast, 'a, T> Visit<'ast, 'a, T> for UnionMemberTypes<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_union_type_definition(
        &self,
        node: &'ast UnionTypeDefinition<'a, T>,
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
        node: &'ast UnionTypeExtension<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        if let Some(member_types) = node.member_types.as_ref() {
            accumulator.extend(self.check_union_member_types(name.0.as_ref(), member_types));
        }
    }
}
