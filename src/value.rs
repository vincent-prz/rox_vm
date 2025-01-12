use std::fmt;

use crate::chunk::Chunk;

#[derive(Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Str(String),
    Function(Function),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Function(function) => match &function.name {
                Some(func_name) => write!(f, "<fn {}>", func_name),
                None => write!(f, "<script>"),
            },
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Number(n) => *n == 0.0,
            Value::Boolean(b) => !b,
            Value::Str(s) => s == "",
            Value::Function(_) => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_falsey()
    }
}

#[derive(Clone, PartialEq)]
pub struct Function {
    arity: u8,
    pub chunk: Chunk,
    pub name: Option<String>,
}

impl Function {
    pub fn new() -> Self {
        Function {
            arity: 0,
            name: None,
            chunk: Chunk::new(),
        }
    }
}
