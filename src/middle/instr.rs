use crate::front::parser::Value;

#[derive(Debug)]
pub enum Instructions {
    Store { value: Value, name: Box<str> },
    Push { value: Value },
}