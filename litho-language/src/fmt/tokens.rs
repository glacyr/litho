use std::borrow::Borrow;
use std::fmt::{Result, Write};

use unindent::unindent;

use crate::ast::ContextValue;
use crate::lex::*;

use super::{macros, Format, Formatter};

macros::format_token!(Name);
macros::format_token!(Punctuator);
macros::format_token!(IntValue);
macros::format_token!(FloatValue);

impl<'a, T> Format for StringValue<'a, T>
where
    T: ContextValue<'a> + Borrow<str>,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        let source = self.as_raw_token().source.borrow();
        formatter.push(source)
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        let source: &str = self.as_raw_token().source.borrow();
        formatter.line()?;
        formatter.push(r#"""""#)?;
        for line in unindent(&source[3..source.len() - 3]).lines() {
            formatter.line()?;
            formatter.push(line)?;
        }
        formatter.line()?;
        formatter.push(r#"""""#)?;
        Ok(())
    }

    fn expands(&self) -> bool {
        let source: &str = self.as_raw_token().source.borrow();
        source.starts_with(r#"""""#)
    }
}
