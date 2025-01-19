use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct SelfReferentialDirectives<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> SelfReferentialDirectives<'ast, 'a, T>
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

        let Some(definition) = self.0.directive_definitions_by_name(ty).next() else {
            return false;
        };

        visited.push(ty);

        for arg in definition
            .arguments_definition
            .iter()
            .flat_map(|def| def.definitions.iter())
        {
            for directive in arg
                .directives
                .iter()
                .flat_map(|dirs| dirs.directives.iter())
            {
                match directive.name.ok() {
                    Some(dir) if self.is_recursive(visited, needle, dir.as_ref()) => return true,
                    _ => {}
                }
            }
        }

        visited.pop();

        false
    }
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for SelfReferentialDirectives<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_directive_definition(
        &self,
        node: &'ast Shared<'a, T, DirectiveDefinition<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let Some(name) = node.name.ok() else { return };

        for arg in node
            .arguments_definition
            .iter()
            .flat_map(|def| def.definitions.iter())
        {
            let mut visited = vec![];

            for directive in arg
                .directives
                .iter()
                .flat_map(|dirs| dirs.directives.iter())
            {
                match directive.name.ok() {
                    Some(dir) if self.is_recursive(&mut visited, name.as_ref(), dir.as_ref()) => {
                        accumulator.push(Diagnostic::self_referential_directive(
                            name.as_ref().to_string(),
                            dir.as_ref().to_string(),
                            directive.span(),
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
}
