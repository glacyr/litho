macro_rules! format_token {
    ($ident:ident) => {
        impl<T> Format for $ident<T>
        where
            T: Borrow<str>,
        {
            fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
            where
                W: Write,
            {
                formatter.push(self.as_raw_token().source.borrow())
            }
        }
    };
}

pub(crate) use format_token;

macro_rules! format_enum {
    ($ident:ident, $($variant:ident),*) => {
        impl<T> Format for $ident<T>
        where
            T: Borrow<str>,
        {
            fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
            where
                W: Write,
            {
                match self {
                    $(
                        $ident::$variant(node) => node.format_collapsed(formatter),
                    )*
                }
            }

            fn format_expanded<W>(&self, formatter: &mut Formatter<W>) -> Result
            where
                W: Write,
            {
                match self {
                    $(
                        $ident::$variant(node) => node.format_expanded(formatter),
                    )*
                }
            }

            fn expands(&self) -> bool {
                match self {
                    $(
                        $ident::$variant(node) => node.expands(),
                    )*
                }
            }

            fn can_expand(&self) -> bool {
                match self {
                    $(
                        $ident::$variant(node) => node.can_expand(),
                    )*
                }
            }
        }
    }
}

pub(crate) use format_enum;

macro_rules! format_definitions {
    ($ident:ident) => {
        impl<T> Format for $ident<T>
        where
            T: Borrow<str>,
        {
            fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
            where
                W: Write,
            {
                formatter.each_page(self.definitions.iter())
            }
        }
    };
}

pub(crate) use format_definitions;

macro_rules! format_unit {
    ($ident:ident) => {
        impl<T> Format for $ident<T>
        where
            T: Borrow<str>,
        {
            fn format_collapsed<W>(&self, formatter: &mut Formatter<W>) -> Result
            where
                W: Write,
            {
                self.0.format(formatter)
            }
        }
    };
}

pub(crate) use format_unit;
