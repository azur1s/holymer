use std::fmt::{Display, Formatter, Result as FmtResult};
use super::ty::Type;

#[derive(Clone, Debug)]
pub enum JSLiteral { Num(i64), Str(String), Bool(bool) }

/// Enum to represent javascript expression
#[derive(Clone, Debug)]
pub enum JSExpr {
    Lit(JSLiteral),
    Sym(String),

    Op(&'static str, Box<Self>, Option<Box<Self>>),

    Call(Box<Self>, Vec<Self>),
    Method(Box<Self>, String, Vec<Self>),
    Lambda {
        args: Vec<(String, Type)>,
        body: Box<Self>,
    },
}

impl Display for JSExpr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            JSExpr::Lit(l) => match l {
                JSLiteral::Num(n)  => write!(f, "{}", n),
                JSLiteral::Str(s)  => write!(f, "'{}'", s),
                JSLiteral::Bool(b) => write!(f, "{}", b),
            },
            JSExpr::Sym(s) => write!(f, "{}", s),

            JSExpr::Op(op, lhs, rhs) => {
                match rhs {
                    Some(rhs) => write!(f, "({} {} {})", lhs, op, rhs),
                    None => write!(f, "({} {})", op, lhs),
                }
            }

            JSExpr::Call(c, args) => {
                write!(f, "{}(", c)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            JSExpr::Method(c, m, args) => {
                write!(f, "{}.{}(", c, m)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            JSExpr::Lambda { args, body } => {
                write!(f, "((")?;
                for (i, (name, _ty)) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", name)?;
                }
                write!(f, ") => {})", body)
            },
        }
    }
}