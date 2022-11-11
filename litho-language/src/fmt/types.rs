use std::fmt::{Result, Write};
use std::ops::Range;
use std::sync::Arc;

use crate::ast::*;

use super::Measurer;

pub struct Shape {
    indent: usize,
    range: Range<usize>,
    line: usize,
    whitespace: bool,
    blank_lines: usize,
}

pub struct Formatter<W> {
    writer: W,
    shape: Shape,
}

impl<W> Formatter<W>
where
    W: Write,
{
    pub fn new(writer: W, line_width: usize) -> Formatter<W> {
        Formatter {
            writer,
            shape: Shape {
                indent: 0,
                range: 0..line_width,
                line: 0,
                whitespace: true,
                blank_lines: 2,
            },
        }
    }

    pub fn squeeze<F>(&mut self, closure: F) -> Result
    where
        F: FnOnce(&mut Formatter<W>) -> Result,
    {
        let whitespace = self.shape.whitespace;
        self.shape.whitespace = true;
        let result = closure(self);
        self.shape.whitespace = whitespace;
        result
    }

    pub fn indent<F>(&mut self, closure: F) -> Result
    where
        F: FnOnce(&mut Formatter<W>) -> Result,
    {
        self.shape.indent += 4;
        self.shape.blank_lines = 0;
        self.line()?;
        self.shape.blank_lines = 2;
        let result = closure(self);
        self.line()?;
        self.shape.indent -= 4;
        self.shape.blank_lines = 0;
        result
    }

    pub fn lines(&mut self, num_lines: usize) -> Result {
        if self.shape.blank_lines >= num_lines {
            return Ok(());
        }

        for _ in 0..num_lines - self.shape.blank_lines {
            self.writer.write_str("\n")?;
        }

        self.shape.range.start = 0;
        self.shape.line += num_lines - self.shape.blank_lines;
        self.shape.blank_lines += num_lines - self.shape.blank_lines;
        self.shape.whitespace = true;

        Ok(())
    }

    pub fn line(&mut self) -> Result {
        self.lines(1)
    }

    pub fn page(&mut self) -> Result {
        self.lines(2)
    }

    pub fn each<I>(&mut self, iter: I) -> Result
    where
        I: Iterator,
        I::Item: Format,
    {
        for item in iter {
            item.format(self)?;
        }

        Ok(())
    }

    pub fn each_comma<I>(&mut self, iter: I) -> Result
    where
        I: Iterator,
        I::Item: Format,
    {
        for (i, item) in iter.enumerate() {
            if i != 0 {
                self.squeeze(|formatter| formatter.push(","))?;
            }

            if Measurer::measure(&item, self.shape.range.len()).is_err() {
                self.line()?;
            }

            item.format(self)?;
        }

        Ok(())
    }

    pub fn each_page<I>(&mut self, iter: I) -> Result
    where
        I: Iterator,
        I::Item: Format,
    {
        for item in iter {
            self.page()?;
            item.format(self)?;
        }

        Ok(())
    }

    pub fn each_line<I>(&mut self, iter: I) -> Result
    where
        I: Iterator,
        I::Item: Format,
    {
        for item in iter {
            self.line()?;
            item.format(self)?;
        }

        Ok(())
    }

    pub fn each_line_comma<I>(&mut self, iter: I) -> Result
    where
        I: Iterator,
        I::Item: Format,
    {
        for item in iter {
            self.line()?;
            item.format(self)?;
            self.squeeze(|formatter| formatter.push(","))?;
        }

        Ok(())
    }

    pub fn push(&mut self, token: &str) -> Result {
        if !self.shape.whitespace {
            self.writer.write_char(' ')?;
        } else if self.shape.range.start == 0 {
            self.writer.write_str(&" ".repeat(self.shape.indent))?;
        }

        self.writer.write_str(token)?;

        self.shape.whitespace = false;
        self.shape.blank_lines = 0;
        self.shape.range.start += token.len();

        Ok(())
    }
}

pub trait Format {
    fn format<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        let expanded = self.expands()
            || (self.can_expand() && Measurer::measure(self, formatter.shape.range.len()).is_err());

        match expanded {
            true => self.format_expanded(formatter),
            false => self.format_collapsed(formatter),
        }
    }

    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write;

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.format_collapsed(formatter)
    }

    fn format_to_string(&self, line_width: usize) -> String {
        let mut string = String::new();
        let _ = self.format(&mut Formatter::new(&mut string, line_width));
        string
    }

    fn can_expand(&self) -> bool {
        false
    }

    fn expands(&self) -> bool {
        false
    }
}

impl<A, B> Format for (A, B)
where
    A: Format,
    B: Format,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.0.format(formatter)?;
        self.1.format(formatter)?;
        Ok(())
    }
}

impl<T> Format for &T
where
    T: Format,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        (*self).format_collapsed(formatter)
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        (*self).format_expanded(formatter)
    }

    fn can_expand(&self) -> bool {
        (*self).can_expand()
    }

    fn expands(&self) -> bool {
        (*self).expands()
    }
}

impl<T> Format for Arc<T>
where
    T: Format,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.as_ref().format_collapsed(formatter)
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        self.as_ref().format_expanded(formatter)
    }

    fn can_expand(&self) -> bool {
        self.as_ref().can_expand()
    }

    fn expands(&self) -> bool {
        self.as_ref().expands()
    }
}

impl<T> Format for Option<T>
where
    T: Format,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        match self {
            Some(value) => value.format_collapsed(formatter),
            None => Ok(()),
        }
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        match self {
            Some(value) => value.format_expanded(formatter),
            None => Ok(()),
        }
    }

    fn expands(&self) -> bool {
        match self {
            Some(value) => value.expands(),
            None => false,
        }
    }

    fn can_expand(&self) -> bool {
        match self {
            Some(value) => value.can_expand(),
            None => false,
        }
    }
}

impl<T> Format for Recoverable<T>
where
    T: Format,
{
    fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        match self {
            Recoverable::Present(present) => present.format_collapsed(formatter),
            Recoverable::Missing(_) => Ok(()),
        }
    }

    fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
    where
        W: Write,
    {
        match self {
            Recoverable::Present(present) => present.format_expanded(formatter),
            Recoverable::Missing(_) => Ok(()),
        }
    }

    fn expands(&self) -> bool {
        match self {
            Recoverable::Present(value) => value.expands(),
            Recoverable::Missing(_) => false,
        }
    }

    fn can_expand(&self) -> bool {
        match self {
            Recoverable::Present(value) => value.can_expand(),
            Recoverable::Missing(_) => false,
        }
    }
}
