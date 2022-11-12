use std::borrow::Borrow;
use std::fmt::{Result, Write};

use crate::ast::*;

use super::{macros, Format, Formatter};

macros::format_definitions!(TypeSystemDocument);

macros::format_enum!(
    TypeSystemDefinition,
    SchemaDefinition,
    TypeDefinition,
    DirectiveDefinition
);

macros::format_definitions!(TypeSystemExtensionDocument);

macros::format_enum!(
    TypeSystemDefinitionOrExtension,
    TypeSystemDefinition,
    TypeSystemExtension
);

macros::format_enum!(TypeSystemExtension, SchemaExtension, TypeExtension);

macros::format_unit!(Description);

impl<T> Format for SchemaDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for RootOperationTypeDefinitions<T>
where
    T: Borrow<str>,
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

impl<T> Format for RootOperationTypeDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for SchemaExtension<T>
where
    T: Borrow<str>,
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

impl<T> Format for ScalarTypeDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for ScalarTypeExtension<T>
where
    T: Borrow<str>,
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

impl<T> Format for ObjectTypeDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for ImplementsInterfaces<T>
where
    T: Borrow<str>,
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

impl<T> Format for FieldsDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for FieldDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for ArgumentsDefinition<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.parens.0.format(formatter)?;

        if self.expands() {
            formatter.indent(|formatter| formatter.each_line_comma(self.definitions.iter()))?;
        } else {
            formatter.squeeze(|formatter| formatter.each_comma(self.definitions.iter()))?;
        }

        self.parens.1.format(formatter)?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.definitions.iter().any(Format::expands)
    }
}

impl<T> Format for InputValueDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for ObjectTypeExtension<T>
where
    T: Borrow<str>,
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

impl<T> Format for InterfaceTypeDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for InterfaceTypeExtension<T>
where
    T: Borrow<str>,
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

impl<T> Format for UnionTypeDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for UnionMemberTypes<T>
where
    T: Borrow<str>,
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

impl<T> Format for UnionTypeExtension<T>
where
    T: Borrow<str>,
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

impl<T> Format for EnumTypeDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for EnumValuesDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for EnumValueDefinition<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        if let Some(description) = self.description.as_ref() {
            formatter.page()?;
            description.format(formatter)?;
        }

        self.enum_value.format(formatter)?;
        self.directives.format(formatter)?;

        Ok(())
    }
}

impl<T> Format for EnumTypeExtension<T>
where
    T: Borrow<str>,
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

impl<T> Format for InputObjectTypeDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for InputFieldsDefinition<T>
where
    T: Borrow<str>,
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

impl<T> Format for InputObjectTypeExtension<T>
where
    T: Borrow<str>,
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

impl<T> Format for DirectiveDefinition<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.page()?;
        self.description.format(formatter)?;

        formatter.line()?;
        self.directive.format(formatter)?;
        formatter.squeeze(|formatter| self.name.format(formatter))?;
        formatter.squeeze(|formatter| self.arguments_definition.format(formatter))?;
        self.repeatable.format(formatter)?;
        self.locations.format(formatter)?;

        Ok(())
    }
}

impl<T> Format for DirectiveLocations<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.on.format(formatter)?;

        if self.expands() {
            formatter.indent(|formatter| {
                for location in self.locations() {
                    formatter.push("|")?;
                    location.format(formatter)?;
                }

                Ok(())
            })?;
        } else {
            for (i, location) in self.locations().enumerate() {
                if i != 0 {
                    formatter.push("|")?;
                    location.format(formatter)?;
                }
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
