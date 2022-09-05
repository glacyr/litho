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

    fn is_equal(&self, other: &Self) -> bool;

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

    fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Variable(lhs), Value::Variable(rhs)) => lhs == rhs,
            (Value::Int(lhs), Value::Int(rhs)) => lhs == rhs,
            (Value::Float(lhs), Value::Float(rhs)) => lhs == rhs,
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Boolean(lhs), Value::Boolean(rhs)) => lhs == rhs,
            (Value::Null, Value::Null) => true,
            (Value::Enum(lhs), Value::Enum(rhs)) => lhs == rhs,
            (Value::List(lhs), Value::List(rhs)) => {
                lhs.len() == rhs.len()
                    && lhs
                        .iter()
                        .zip(rhs.iter())
                        .all(|(lhs, rhs)| lhs.is_equal(rhs))
            }
            (Value::Object(lhs), Value::Object(rhs)) => {
                lhs.len() == rhs.len()
                    && lhs.iter().all(|(key, lhs)| {
                        rhs.get(key.as_ref()).map(|rhs| lhs.is_equal(rhs)) == Some(true)
                    })
            }
            _ => false,
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

    fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(lhs), Some(rhs)) => lhs.is_equal(rhs),
            (None, None) => true,
            _ => false,
        }
    }

    fn is_null(&self) -> bool {
        match self {
            Some(value) => value.is_null(),
            None => true,
        }
    }
}
