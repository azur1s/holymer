use std::fmt::{Display, Formatter, Result as FmtResult};
use crate::trans::ty::Type;

#[derive(Clone, Debug)]
pub enum JSLiteral { Num(i64), Str(String), Bool(bool), Undefined }

/// Enum to represent javascript expression
#[derive(Clone, Debug)]
pub enum JSExpr {
    Lit(JSLiteral),
    Sym(String),
    Array(Vec<Self>),

    Op(&'static str, Box<Self>, Option<Box<Self>>),

    Call(Box<Self>, Vec<Self>),
    Method(Box<Self>, String),
    Lambda {
        args: Vec<String>,
        body: Vec<Self>,
    },
    If {
        cond: Box<Self>,
        t: Box<Self>,
        f: Box<Self>,
    },
    Defines(Vec<(String, Self)>),
    Return(Box<Self>),
}

#[derive(Clone, Debug)]
pub enum JSStmt {
    Expr(JSExpr),
    Func {
        name: String,
        args: Vec<String>,
        ret: Type,
        body: JSExpr,
    },
}

impl Display for JSExpr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            JSExpr::Lit(l) => match l {
                JSLiteral::Num(n)    => write!(f, "{}", n),
                JSLiteral::Str(s)    => write!(f, "'{}'", s),
                JSLiteral::Bool(b)   => write!(f, "{}", b),
                JSLiteral::Undefined => write!(f, "undefined"),
            },
            JSExpr::Sym(s) => write!(f, "{}", s),
            JSExpr::Array(v) => {
                write!(f, "[")?;
                for (i, e) in v.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            },

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
            JSExpr::Method(c, m) => write!(f, "{}.{}", c, m),
            JSExpr::Lambda { args, body } => {
                write!(f, "((")?;
                for (i, name) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", name)?;
                }
                // write!(f, ") => {})", body)
                if body.len() == 1 {
                    write!(f, ") => {})", body[0])
                } else {
                    write!(f, ") => {{")?;
                    for e in body {
                        write!(f, "{};", e)?;
                    }
                    write!(f, "}})")
                }
            },
            JSExpr::If { cond, t, f: fe } => {
                write!(f, "({} ? {} : {})", cond, t, fe)
            },
            JSExpr::Defines(vs) => {
                write!(f, "let [")?;
                for (i, (name, _)) in vs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", name)?;
                }
                write!(f, "] = [")?;
                for (i, (_, expr)) in vs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, "]")
            }
            JSExpr::Return(e) => write!(f, "return {}", e),
        }
    }
}

impl Display for JSStmt {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            JSStmt::Expr(e) => write!(f, "{}", e),
            JSStmt::Func { name, args, ret: _, body } => {
                // const name = (args) => body;
                write!(f, "const {} = (", name)?;
                for (i, name) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", name)?;
                }
                write!(f, ") => {};", body)
            },
        }
    }
}