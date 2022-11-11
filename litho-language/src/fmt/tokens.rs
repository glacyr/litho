use std::borrow::Borrow;
use std::fmt::{Result, Write};

use crate::lex::*;

use super::{macros, Format, Formatter};

macros::format_token!(Name);
macros::format_token!(Punctuator);
macros::format_token!(StringValue);
macros::format_token!(IntValue);
macros::format_token!(FloatValue);
