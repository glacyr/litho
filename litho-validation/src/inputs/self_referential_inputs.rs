use std::hash::Hash;

use litho_language::ast::*;
use litho_types::Database;

use crate::Error;

pub struct SelfReferentialInputs<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> SelfReferentialInputs<'a, T>
where
    T: Eq + Hash,
{
    pub fn is_recursive(&self, visited: &mut Vec<&'a T>, needle: &T, ty: &'a T) -> bool {
        if needle == ty {
            return true;
        }

        if visited.contains(&ty) {
            return false;
        }

        visited.push(ty);

        for field in self.0.input_field_definitions(ty) {
            match field.ty.ok() {
                Some(Type::NonNull(ty)) => match &ty.ty {
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

impl<'a, T> Visit<'a, T> for SelfReferentialInputs<'a, T>
where
    T: Eq + Hash,
{
    type Accumulator = Vec<Error<'a, T>>;

    fn visit_input_object_type_definition(
        &self,
        node: &'a InputObjectTypeDefinition<T>,
        accumulator: &mut Self::Accumulator,
    ) {
        let name = match node.name.ok() {
            Some(name) => name,
            None => return,
        };

        for field in node
            .fields_definition
            .iter()
            .flat_map(|def| def.definitions.iter())
        {
            let mut visited = vec![];

            match field.ty.ok() {
                Some(Type::NonNull(ty)) => match &ty.ty {
                    Type::Named(ty)
                        if self.is_recursive(&mut visited, name.as_ref(), ty.0.as_ref()) =>
                    {
                        accumulator.push(Error::SelfReferentialInputType {
                            name: name.as_ref(),
                            field: field.name.as_ref(),
                            ty: ty.0.as_ref(),
                            span: field.name.span(),
                        })
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
