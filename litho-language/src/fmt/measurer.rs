use std::fmt::{Error, Result, Write};

use super::{Format, Formatter};

pub struct Measurer(usize, usize);

impl Measurer {
    pub fn measure<T>(node: &T, width: usize) -> Result
    where
        T: Format + ?Sized,
    {
        let mut formatter = Formatter::new(Measurer(0, width), width);
        node.format_collapsed(&mut formatter)
    }
}

impl Write for Measurer {
    fn write_str(&mut self, s: &str) -> Result {
        if s.contains("\r") || s.contains("\n") {
            self.0 += self.1;
        } else {
            self.0 += s.len();
        }

        if self.0 > self.1 {
            return Err(Error);
        } else {
            return Ok(());
        }
    }
}
