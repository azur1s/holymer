use std::fmt;

use crate::front::parser::Value;

#[derive(Debug)]
pub enum Instructions {
    Store { value: Value, name: Box<str> },
    Push { value: Value },
}

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instructions::Store { value, name } => write!(f, "store {} {}", value, name),
            Instructions::Push { value } => write!(f, "push {}", value),
        }
    }
}