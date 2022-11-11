use std::borrow::Borrow;
use std::fmt::{Result, Write};

use crate::ast::*;

use super::{macros, Format, Formatter};

macros::format_enum!(
    ExecutableDefinition,
    OperationDefinition,
    FragmentDefinition
);

impl<T> Format for OperationDefinition<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.ty.format(formatter)?;
        self.name.format(formatter)?;
        formatter.squeeze(|formatter| self.variable_definitions.format(formatter))?;
        self.directives.format(formatter)?;
        self.selection_set.format(formatter)?;
        Ok(())
    }
}

impl<T> Format for SelectionSet<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.braces.0.format(formatter)?;

        if self.expands() {
            formatter.indent(|formatter| formatter.each_line(self.selections.iter()))?;
        } else {
            formatter.indent(|formatter| formatter.each(self.selections.iter()))?;
        }

        self.braces.1.format(formatter)?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.selections.iter().any(Format::expands)
    }
}

macros::format_enum!(Selection, Field, FragmentSpread, InlineFragment);

impl<T> Format for Field<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.alias.format(formatter)?;
        self.name.format(formatter)?;
        formatter.squeeze(|formatter| self.arguments.format(formatter))?;
        self.directives.format(formatter)?;
        self.selection_set.format(formatter)?;
        Ok(())
    }

    fn expands(&self) -> bool {
        self.alias.is_some() || self.selection_set.is_some()
    }
}

impl<T> Format for Alias<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.name.format(formatter)?;
        self.colon.format(formatter)?;
        Ok(())
    }
}

impl<T> Format for FragmentSpread<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.dots.format(formatter)?;
        self.fragment_name.format(formatter)?;
        self.directives.format(formatter)?;
        Ok(())
    }

    fn expands(&self) -> bool {
        true
    }
}

impl<T> Format for FragmentDefinition<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.fragment.format(formatter)?;
        self.fragment_name.format(formatter)?;
        self.type_condition.format(formatter)?;
        self.directives.format(formatter)?;
        self.selection_set.format(formatter)?;
        Ok(())
    }

    fn expands(&self) -> bool {
        true
    }
}

impl<T> Format for TypeCondition<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.on.format(formatter)?;
        self.named_type.format(formatter)?;
        Ok(())
    }
}

impl<T> Format for InlineFragment<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.dots.format(formatter)?;
        self.type_condition.format(formatter)?;
        self.directives.format(formatter)?;
        self.selection_set.format(formatter)?;
        Ok(())
    }

    fn expands(&self) -> bool {
        true
    }
}

impl<T> Format for VariableDefinitions<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.parens.0.format(formatter)?;

        if self.expands() {
            formatter
                .indent(|formatter| formatter.each_line_comma(self.variable_definitions.iter()))?;
        } else {
            formatter
                .squeeze(|formatter| formatter.each_comma(self.variable_definitions.iter()))?;
        }

        formatter.squeeze(|formatter| self.parens.1.format(formatter))?;

        Ok(())
    }

    fn expands(&self) -> bool {
        self.variable_definitions.iter().any(Format::expands)
    }
}

impl<T> Format for VariableDefinition<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.variable.format(formatter)?;
        formatter.squeeze(|formatter| self.colon.format(formatter))?;
        self.ty.format(formatter)?;
        self.default_value.format(formatter)?;
        self.directives.format(formatter)?;
        Ok(())
    }

    fn expands(&self) -> bool {
        self.default_value.expands()
    }
}

impl<T> Format for DefaultValue<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.eq.format(formatter)?;
        self.value.format(formatter)?;
        Ok(())
    }

    fn expands(&self) -> bool {
        self.value.expands()
    }
}

macros::format_enum!(Type, Named, List, NonNull);

impl<T> Format for ListType<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.brackets.0.format(formatter)?;
        formatter.squeeze(|formatter| self.ty.format(formatter))?;
        formatter.squeeze(|formatter| self.brackets.1.format(formatter))?;
        Ok(())
    }
}

impl<T> Format for NonNullType<T>
where
    T: Borrow<str>,
{
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.ty.format(formatter)?;
        formatter.squeeze(|formatter| self.bang.format(formatter))?;
        Ok(())
    }
}
