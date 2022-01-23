use std::rc::Rc;

use crate::util::unescape;

#[derive(Debug, Clone)]
pub enum Type {
    Null,
    Bool(bool),
    Number(i64),
    Str(String),
    Symbol(String),
    List(Rc<Vec<Type>>, Rc<Type>),
    Vector(Rc<Vec<Type>>, Rc<Type>),
    // Function(fn(Arguments) -> Return, Rc<Type>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Null => write!(f, "Null"),
            Type::Bool(b) => write!(f, "{}", b),
            Type::Number(n) => write!(f, "{}", n),
            Type::Str(s) => write!(f, "\"{}\"", unescape(s.to_string())),
            Type::Symbol(s) => write!(f, "{}", s),
            Type::List(l, _) => write!(f, "({})", l.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join(" ")),
            Type::Vector(l, _) => write!(f, "[{}]", l.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join(", ")),
            // Type::Function(func, _) => write!(f, "<{:?}>", func),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ErrorString(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ErrorString(s) => write!(f, "{}", s),
        }
    }
}

// pub type Arguments = Vec<Type>;
pub type Return = Result<Type, Error>;

#[macro_export]
macro_rules! list {
    ($seq:expr) => {{
        List(Rc::new($seq),Rc::new(Null))
    }};
    [$($args:expr),*] => {{
        let v: Vec<Type> = vec![$($args),*];
        List(Rc::new(v),Rc::new(Null))
    }}
}

#[macro_export]
macro_rules! vector {
    ($seq:expr) => {{
        Vector(Rc::new($seq), Rc::new(Null))
    }};
    [$($args:expr),*] => {{
        let v: Vec<Type> = vec![$($args),*];
        Vector(Rc::new(v), Rc::new(Null))
    }}
}