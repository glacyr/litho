use std::fmt::Display;

use litho_language::ast::*;

pub struct Printer;

impl Printer {
    pub fn pretty_print_field<T>(definition: &FieldDefinition<T>) -> String
    where
        T: Display,
    {
        format!(
            "{}{}: {}",
            definition.name,
            Printer::pretty_print_arguments_definition(definition.arguments_definition.as_deref()),
            definition
                .ty
                .ok()
                .map(ToString::to_string)
                .unwrap_or("...".to_owned())
        )
    }

    pub fn print_arguments_definition<T>(definition: &ArgumentsDefinition<T>) -> String
    where
        T: Display,
    {
        format!(
            "({})",
            definition
                .definitions
                .iter()
                .map(|arg| format!(
                    "{}: {}",
                    arg.name,
                    arg.ty
                        .ok()
                        .map(|ty| ty.to_string())
                        .unwrap_or("...".to_owned())
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn pretty_print_arguments_definition<T>(
        arguments_definition: Option<&ArgumentsDefinition<T>>,
    ) -> String
    where
        T: Display,
    {
        match arguments_definition {
            Some(arguments_definition) => format!(
                "(\n{}\n)",
                arguments_definition
                    .definitions
                    .iter()
                    .map(|arg| {
                        format!(
                            "    {}: {}",
                            arg.name,
                            arg.ty
                                .ok()
                                .map(ToString::to_string)
                                .unwrap_or("...".to_owned())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            None => String::new(),
        }
    }

    pub fn snippy_print_arguments_definition<T>(
        arguments_definition: Option<&ArgumentsDefinition<T>>,
    ) -> String
    where
        T: Display,
    {
        match arguments_definition {
            Some(arguments_definition) => format!(
                "({})",
                arguments_definition
                    .definitions
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        format!(
                            "{}: ${{{}:{}}}",
                            arg.name,
                            i + 1,
                            arg.ty
                                .ok()
                                .map(ToString::to_string)
                                .unwrap_or("...".to_owned())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None => String::new(),
        }
    }
}
