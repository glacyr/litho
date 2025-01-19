use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct SelfReferentialInputs<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> SelfReferentialInputs<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash,
{
    pub fn is_recursive(&self, visited: &mut Vec<&'ast T>, needle: &T, ty: &'ast T) -> bool {
        if needle == ty {
            return true;
        }

        if visited.contains(&ty) {
            return false;
        }

        visited.push(ty);

        for field in self.0.input_value_definitions(ty) {
            match field.ty.ok().map(AsRef::as_ref) {
                Some(Type::NonNull(ty)) => match ty.ty.as_ref() {
                    Type::Named(ty) if self.is_recursive(visited, needle, ty.0.as_ref()) => {
                        return true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        visited.pop();

        false
    }
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for SelfReferentialInputs<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_input_object_type_definition(
        &self,
        node: &'ast InputObjectTypeDefinition<'a, T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        for field in node
            .fields_definition
            .iter()
            .flat_map(|def| def.definitions.iter())
        {
            let mut visited = vec![];

            match field.ty.ok().map(AsRef::as_ref) {
                Some(Type::NonNull(ty)) => match ty.ty.as_ref() {
                    Type::Named(ty)
                        if self.is_recursive(&mut visited, name.as_ref(), ty.0.as_ref()) =>
                    {
                        accumulator.push(Diagnostic::self_referential_input_type(
                            name.as_ref().to_string(),
                            field.name.as_ref().to_string(),
                            ty.0.as_ref().to_string(),
                            field.name.span(),
                        ));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
