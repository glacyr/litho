use std::borrow::Borrow;
use std::fmt::{Result, Write};

use crate::ast::*;

use super::{macros, Format, Formatter};

macros::format_definitions!(Document);

macros::format_enum!(
    Definition,
    ExecutableDefinition,
    TypeSystemDefinitionOrExtension
);

macros::format_enum!(OperationType, Query, Mutation, Subscription);

impl<T> Format for Arguments<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.parens.0.format(formatter)?;
        formatter.squeeze(|formatter| formatter.each(self.items.iter()))?;
        formatter.squeeze(|formatter| self.parens.1.format(formatter))?;
        Ok(())
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.parens.0.format(formatter)?;
        formatter.indent(|formatter| formatter.each_line_comma(self.items.iter()))?;
        formatter.squeeze(|formatter| self.parens.1.format(formatter))?;
        Ok(())
    }

    fn expands(&self) -> bool {
        self.items.iter().any(Format::expands)
    }

    fn can_expand(&self) -> bool {
        true
    }
}

impl<T> Format for Argument<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.name.format(formatter)?;
        formatter.squeeze(|formatter| self.colon.format(formatter))?;
        self.value.format(formatter)?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.value.expands()
    }
}

macros::format_enum!(
    Value,
    IntValue,
    FloatValue,
    StringValue,
    BooleanValue,
    NullValue,
    EnumValue,
    Variable,
    ListValue,
    ObjectValue
);

macros::format_enum!(BooleanValue, True, False);
macros::format_unit!(NullValue);
macros::format_unit!(EnumValue);

impl<T> Format for ListValue<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.brackets.0.format(formatter)?;
        formatter.squeeze(|formatter| formatter.each_comma(self.values.iter()))?;
        formatter.squeeze(|formatter| self.brackets.1.format(formatter))?;

        Ok(())
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.brackets.0.format(formatter)?;

        if self.expands() {
            formatter.indent(|formatter| formatter.each_line_comma(self.values.iter()))?;
        } else {
            formatter.indent(|formatter| formatter.each_comma(self.values.iter()))?;
        }

        formatter.squeeze(|formatter| self.brackets.1.format(formatter))?;

        Ok(())
    }

    fn can_expand(&self) -> bool {
        true
    }

    fn expands(&self) -> bool {
        self.values.iter().any(Format::expands)
    }
}

impl<T> Format for ObjectValue<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.braces.0.format(formatter)?;
        formatter.each(self.object_fields.iter())?;
        self.braces.1.format(formatter)?;

        Ok(())
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.braces.0.format(formatter)?;
        formatter.indent(|formatter| formatter.each_line_comma(self.object_fields.iter()))?;
        self.braces.1.format(formatter)?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.object_fields.len() > 1
    }
}

impl<T> Format for ObjectField<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.name.format(formatter)?;
        formatter.squeeze(|formatter| self.colon.format(formatter))?;
        self.value.format(formatter)?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.value.expands()
    }
}

impl<T> Format for Variable<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.dollar.format(formatter)?;
        formatter.squeeze(|formatter| self.name.format(formatter))?;

        Ok(())
    }
}

macros::format_unit!(NamedType);

impl<T> Format for Directives<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.each(self.directives.iter())
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        formatter.indent(|formatter| formatter.each_line(self.directives.iter()))
    }

    fn expands(&self) -> bool {
        self.directives.len() > 3
    }

    fn can_expand(&self) -> bool {
        true
    }
}

impl<T> Format for Directive<T>
where
    T: Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.at.format(formatter)?;
        formatter.squeeze(|formatter| self.name.format(formatter))?;
        formatter.squeeze(|formatter| self.arguments.format(formatter))?;

        Ok(())
    }
}
