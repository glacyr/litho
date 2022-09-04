use graphql_parser::schema::{Text, Value};

#[derive(Debug)]
pub enum ValueType {
    Variable,
    Int,
    Float,
    String,
    Boolean,
    Null,
    Enum,
    List,
    Object,
}

impl ValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValueType::Variable => "VARIABLE",
            ValueType::Int => "INT",
            ValueType::Float => "FLOAT",
            ValueType::String => "STRING",
            ValueType::Boolean => "BOOLEAN",
            ValueType::Null => "NULL",
            ValueType::Enum => "ENUM",
            ValueType::List => "LIST",
            ValueType::Object => "OBJECT",
        }
    }
}

pub trait ValueExt {
    fn ty(&self) -> ValueType;

    fn is_null(&self) -> bool;
}

impl<'a, T> ValueExt for Value<'a, T>
where
    T: Text<'a>,
{
    fn ty(&self) -> ValueType {
        match self {
            Value::Variable(_) => ValueType::Variable,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Null => ValueType::Null,
            Value::Enum(_) => ValueType::Enum,
            Value::List(_) => ValueType::List,
            Value::Object(_) => ValueType::Object,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl<'a, T> ValueExt for Option<Value<'a, T>>
where
    T: Text<'a>,
{
    fn ty(&self) -> ValueType {
        match self {
            Some(value) => value.ty(),
            None => ValueType::Null,
        }
    }

    fn is_null(&self) -> bool {
        match self {
            Some(value) => value.is_null(),
            None => true,
        }
    }
}
