use std::{cell::RefCell, fmt, rc::Rc};

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
            Value::Function(function) => write!(f, "<fn {}>", function.name),
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
    pub arity: usize,
    // perf: we need mutability only when compiling, not during runtime
    // try to remove refcell during runtime
    pub chunk: Rc<RefCell<Chunk>>,
    pub name: String,
}

impl Function {
    pub fn new(name: String, arity: usize) -> Self {
        Function {
            arity,
            name,
            chunk: Rc::new(RefCell::new(Chunk::new())),
        }
    }
}
