use std::rc::Rc;

use crate::util::unescape;

#[derive(Debug, Clone)]
pub enum Expr {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Symbol(String),
    List(Rc<Vec<Expr>>, Rc<Expr>),
    Vector(Rc<Vec<Expr>>, Rc<Expr>),
    // Function(fn(Arguments) -> Return, Rc<Expr>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Null => write!(f, "Null"),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::String(s) => write!(f, "{}", unescape(s.to_string())),
            Expr::Symbol(s) => write!(f, "{}", s),
            Expr::List(l, _) => {
                write!(f, "(")?;
                for e in l.iter() {
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Expr::Vector(l, _) => {
                write!(f, "[")?;
                for e in l.iter() {
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ErrorString(String),
}

// pub type Arguments = Vec<Expr>;
pub type Return = Result<Expr, Error>;

#[macro_export]
macro_rules! list {
    ($seq:expr) => {{
        List(Rc::new($seq),Rc::new(Null))
    }};
    [$($args:expr),*] => {{
        let v: Vec<Expr> = vec![$($args),*];
        List(Rc::new(v),Rc::new(Null))
    }}
}

#[macro_export]
macro_rules! vector {
    ($seq:expr) => {{
        Vector(Rc::new($seq), Rc::new(Null))
    }};
    [$($args:expr),*] => {{
        let v: Vec<Expr> = vec![$($args),*];
        Vector(Rc::new(v), Rc::new(Null))
    }}
}