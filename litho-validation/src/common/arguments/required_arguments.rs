use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct RequiredArguments<'a, T>(pub &'a Database<T>)
where
    T: Eq + Hash;

impl<'a, T> RequiredArguments<'a, T>
where
    T: Eq + Hash + ToString,
{
    fn check_arguments(
        &self,
        name: &Name<T>,
        arguments: Option<&Arc<Arguments<T>>>,
        definition: &Arc<ArgumentsDefinition<T>>,
        accumulator: &mut Vec<Diagnostic<Span>>,
    ) {
        let names = arguments
            .into_iter()
            .flat_map(|arguments| arguments.items.iter())
            .map(|argument| argument.name.as_ref())
            .collect::<HashSet<_>>();

        for definition in definition.definitions.iter() {
            if !names.contains(definition.name.as_ref()) {
                match definition.ty.ok() {
                    Some(ty) if ty.is_required() => {
                        accumulator.push(Diagnostic::missing_required_argument(
                            definition.name.as_ref().to_string(),
                            definition.ty.to_string(),
                            name.span(),
                        ))
                    }
                    Some(_) | None => {}
                }
            }
        }
    }
}

impl<'a, T> Visit<'a, T> for RequiredArguments<'a, T>
where
    T: Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_field(&self, node: &'a Arc<Field<T>>, accumulator: &mut Self::Accumulator) {
        let definition = self.0.inference.arguments_definition_for_field(node);

        if let Some((name, definition)) = node.name.ok().zip(definition) {
            self.check_arguments(name, node.arguments.as_ref(), definition, accumulator);
        }
    }
}
