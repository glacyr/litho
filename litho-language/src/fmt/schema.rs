use std::borrow::Borrow;
use std::fmt::{Result, Write};

use crate::ast::*;

use super::{macros, Format, Formatter};

macros::format_definitions!(TypeSystemDocument);

macros::format_enum!(
    TypeSystemDefinition,
    SchemaDefinition,
    TypeDefinition,
    DirectiveDefinition,
    Error
);

macros::format_definitions!(TypeSystemExtensionDocument);

macros::format_enum!(
    TypeSystemDefinitionOrExtension,
    TypeSystemDefinition,
    TypeSystemExtension
);

macros::format_enum!(TypeSystemExtension, SchemaExtension, TypeExtension, Error);

macros::format_unit!(Description);

impl<'a, T> Format for SchemaDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.schema.format(formatter)?;
        self.directives.format(formatter)?;
        self.type_definitions.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for RootOperationTypeDefinitions<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.braces.0.format(formatter)?;
        formatter
            .indent(|formatter| formatter.each_line(self.definitions.ok().into_iter().flatten()))?;
        self.braces.1.format(formatter)?;
        Ok(())
    }
}

impl<'a, T> Format for RootOperationTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.operation_type.format(formatter)?;
        formatter.squeeze(|formatter| self.colon.format(formatter))?;
        self.named_type.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for SchemaExtension<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.extend_schema.format(formatter)?;
        self.directives.format(formatter)?;
        self.type_definitions.format(formatter)?;

        Ok(())
    }
}

macros::format_enum!(
    TypeDefinition,
    ScalarTypeDefinition,
    ObjectTypeDefinition,
    InterfaceTypeDefinition,
    UnionTypeDefinition,
    EnumTypeDefinition,
    InputObjectTypeDefinition
);

macros::format_enum!(
    TypeExtension,
    ScalarTypeExtension,
    ObjectTypeExtension,
    InterfaceTypeExtension,
    UnionTypeExtension,
    EnumTypeExtension,
    InputObjectTypeExtension
);

impl<'a, T> Format for ScalarTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.scalar.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        Ok(())
    }
}

impl<'a, T> Format for ScalarTypeExtension<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.extend_scalar.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        Ok(())
    }
}

impl<'a, T> Format for ObjectTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.ty.format(formatter)?;
        self.name.format(formatter)?;
        self.implements_interfaces.format(formatter)?;
        self.directives.format(formatter)?;
        self.fields_definition.format(formatter)?;
        Ok(())
    }
}

impl<'a, T> Format for ImplementsInterfaces<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.implements.format(formatter)?;

        for (i, ty) in self.named_types().enumerate() {
            if i != 0 {
                formatter.push("&")?;
            }

            ty.format(formatter)?;
        }

        Ok(())
    }
}

impl<'a, T> Format for FieldsDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.braces.0.format(formatter)?;
        formatter.indent(|formatter| formatter.each_line(self.definitions.iter()))?;
        self.braces.1.format(formatter)?;
        Ok(())
    }
}

impl<'a, T> Format for FieldDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        if let Some(description) = self.description.as_ref() {
            formatter.page()?;
            description.format(formatter)?;
        }

        formatter.line()?;
        self.name.format(formatter)?;
        formatter.squeeze(|formatter| self.arguments_definition.format(formatter))?;
        formatter.squeeze(|formatter| self.colon.format(formatter))?;
        self.ty.format(formatter)?;
        self.directives.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for ArgumentsDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.parens.0.format(formatter)?;
        formatter.squeeze(|formatter| formatter.each_comma(self.definitions.iter()))?;
        self.parens.1.format(formatter)?;

        Ok(())
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.parens.0.format(formatter)?;
        formatter.indent(|formatter| formatter.each_line_comma(self.definitions.iter()))?;
        self.parens.1.format(formatter)?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.definitions.iter().any(Format::expands)
    }

    fn can_expand(&self) -> bool {
        true
    }
}

impl<'a, T> Format for InputValueDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        if let Some(description) = self.description.as_ref() {
            formatter.page()?;
            description.format(formatter)?;
        }

        formatter.line()?;
        self.name.format(formatter)?;
        formatter.squeeze(|formatter| self.colon.format(formatter))?;
        self.ty.format(formatter)?;
        self.default_value.format(formatter)?;
        self.directives.format(formatter)?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.description.is_some() || self.default_value.is_some() || self.directives.is_some()
    }
}

impl<'a, T> Format for ObjectTypeExtension<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.extend_type.format(formatter)?;
        self.name.format(formatter)?;
        self.implements_interfaces.format(formatter)?;
        self.directives.format(formatter)?;
        self.fields_definition.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for InterfaceTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.interface.format(formatter)?;
        self.name.format(formatter)?;
        self.implements_interfaces.format(formatter)?;
        self.directives.format(formatter)?;
        self.fields_definition.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for InterfaceTypeExtension<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.extend_interface.format(formatter)?;
        self.name.format(formatter)?;
        self.implements_interfaces.format(formatter)?;
        self.directives.format(formatter)?;
        self.fields_definition.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for UnionTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.union_kw.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        self.member_types.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for UnionMemberTypes<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.eq.format(formatter)?;

        if self.expands() {
            formatter.indent(|formatter| {
                for ty in self.named_types() {
                    formatter.line()?;
                    formatter.push("|")?;
                    ty.format(formatter)?;
                }

                Ok(())
            })?;
        } else {
            for (i, ty) in self.named_types().enumerate() {
                if i != 0 {
                    formatter.push("|")?;
                }

                ty.format(formatter)?;
            }
        }

        Ok(())
    }

    fn expands(&self) -> bool {
        self.named_types().count() > 3
    }
}

impl<'a, T> Format for UnionTypeExtension<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.extend_union.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        self.member_types.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for EnumTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.enum_kw.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        self.values_definition.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for EnumValuesDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.braces.0.format(formatter)?;
        formatter.indent(|formatter| formatter.each_line(self.definitions.iter()))?;
        self.braces.1.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for EnumValueDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        if let Some(description) = self.description.as_ref() {
            formatter.page()?;
            description.format(formatter)?;
        }

        formatter.line()?;
        self.enum_value.format(formatter)?;
        self.directives.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for EnumTypeExtension<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.extend_enum.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        self.values_definition.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for InputObjectTypeDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.input.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        self.fields_definition.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for InputFieldsDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.braces.0.format(formatter)?;
        formatter.indent(|formatter| formatter.each_line(self.definitions.iter()))?;
        self.braces.1.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for InputObjectTypeExtension<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.extend_input.format(formatter)?;
        self.name.format(formatter)?;
        self.directives.format(formatter)?;
        self.fields_definition.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for DirectiveDefinition<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.directive.format(formatter)?;
        self.at.format(formatter)?;
        formatter.squeeze(|formatter| self.name.format(formatter))?;
        formatter.squeeze(|formatter| self.arguments_definition.format(formatter))?;
        self.repeatable.format(formatter)?;
        self.locations.format(formatter)?;

        Ok(())
    }
}

impl<'a, T> Format for DirectiveLocations<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.on.format(formatter)?;

        if self.expands() {
            formatter.indent(|formatter| {
                for location in self.locations() {
                    formatter.line()?;
                    formatter.push("|")?;
                    location.format(formatter)?;
                }

                Ok(())
            })?;
        } else {
            for (i, location) in self.locations().enumerate() {
                if i != 0 {
                    formatter.push("|")?;
                }
                location.format(formatter)?;
            }
        }

        Ok(())
    }

    fn expands(&self) -> bool {
        self.locations().count() > 3
    }
}

macros::format_enum!(
    DirectiveLocation,
    ExecutableDirectiveLocation,
    TypeSystemDirectiveLocation
);

macros::format_enum!(
    ExecutableDirectiveLocation,
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition
);

macros::format_enum!(
    TypeSystemDirectiveLocation,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition
);
