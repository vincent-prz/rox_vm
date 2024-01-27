use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Str(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Number(n) => *n == 0.0,
            Value::Boolean(b) => !b,
            Value::Str(s) => s == "",
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_falsey()
    }
}
