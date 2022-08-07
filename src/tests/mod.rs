use unindent::unindent;

use kono_schema::{Emit, Schema};

mod impls;

pub fn test_eq<S>(definition: S, expected: &str)
where
    S: Schema,
{
    assert_eq!(
        definition
            .schema()
            .emit()
            .format(&Default::default())
            .trim(),
        unindent(expected).trim()
    );
}
