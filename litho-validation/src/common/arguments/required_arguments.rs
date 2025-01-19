use std::collections::HashSet;
use std::hash::Hash;

use litho_diagnostics::Diagnostic;
use litho_language::ast::*;
use litho_types::Database;

pub struct RequiredArguments<'ast, 'a, T>(pub &'ast Database<'a, T>)
where
    T: ContextValue<'a> + Eq + Hash;

impl<'ast, 'a, T> RequiredArguments<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    fn check_arguments(
        &self,
        name: &Name<'a, T>,
        arguments: Option<&Shared<'a, T, Arguments<'a, T>>>,
        definition: &Shared<'a, T, ArgumentsDefinition<'a, T>>,
        accumulator: &mut Vec<Diagnostic<Span>>,
    ) {
        let names = arguments
            .into_iter()
            .flat_map(|arguments| arguments.items.iter())
            .map(|argument| argument.name.as_ref())
            .collect::<HashSet<_>>();

        for definition in definition.definitions.iter() {
            if !names.contains(definition.name.as_ref()) {
                if definition.is_required() {
                    accumulator.push(Diagnostic::missing_required_argument(
                        definition.name.as_ref().to_string(),
                        definition
                            .ty
                            .ok()
                            .map(|ty| ty.to_string())
                            .unwrap_or("(unknown)".to_owned()),
                        name.span(),
                    ))
                }
            }
        }
    }
}

impl<'ast, 'a, T> Visit<'ast, 'a, T> for RequiredArguments<'ast, 'a, T>
where
    T: ContextValue<'a> + Eq + Hash + ToString,
{
    type Accumulator = Vec<Diagnostic<Span>>;

    fn visit_field(
        &self,
        node: &'ast Shared<'a, T, Field<'a, T>>,
        accumulator: &mut Self::Accumulator,
    ) {
        let definition = self.0.inference.arguments_definition_for_field(node);

        if let Some((name, definition)) = node.name.ok().zip(definition) {
            self.check_arguments(name, node.arguments.as_ref(), definition, accumulator);
        }
    }
}
