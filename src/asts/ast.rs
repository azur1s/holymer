use std::fmt::{Display, Formatter, Result as FmtResult};
use crate::trans::ty::Type;

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
}

#[derive(Clone, Debug)]
pub enum Literal {
    Num(i64), Str(String), Bool(bool),
}

/// Enum to represent internal expression
#[derive(Clone, Debug)]
pub enum Expr {
    Lit(Literal),
    Sym(String),
    Vec(Vec<Self>),

    UnaryOp(UnaryOp, Box<Self>),
    BinaryOp(BinaryOp, Box<Self>, Box<Self>),

    Call(Box<Self>, Vec<Self>),
    Lambda {
        args: Vec<(String, Type)>,
        body: Vec<Self>,
    },
    Return(Box<Self>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Expr(Expr),
    Let(Vec<(String, Type, Expr)>),
    Func {
        name: String,
        args: Vec<(String, Type)>,
        ret: Type,
        body: Expr,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Expr::Lit(l) => match l {
                Literal::Num(n)  => write!(f, "{}", n),
                Literal::Str(s)  => write!(f, "\"{}\"", s),
                Literal::Bool(b) => write!(f, "{}", b),
            },
            Expr::Sym(s) => write!(f, "{}", s),
            Expr::Vec(v) => {
                write!(f, "[")?;
                for (i, e) in v.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            },

            Expr::UnaryOp(op, e)       => write!(f, "({} {})", format!("{:?}", op).to_lowercase(), e),
            Expr::BinaryOp(op, e1, e2) => write!(f, "({} {} {})", format!("{:?}", op).to_lowercase(), e1, e2),

            Expr::Call(c, args) => {
                write!(f, "({}", c)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            },
            Expr::Lambda { args, body } => {
                write!(f, "(lambda ")?;
                for (name, ty) in args {
                    write!(f, "[{} {}]", name, ty)?;
                }
                if body.len() == 1 {
                    write!(f, " {})", body[0])
                } else {
                    write!(f, " (do")?;
                    for e in body {
                        write!(f, " {}", e)?;
                    }
                    write!(f, "))")
                }
            },
            Expr::Return(e) => write!(f, "(return {})", e),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Stmt::Expr(e) => write!(f, "{}", e),
            Stmt::Let(vars) => {
                write!(f, "(let")?;
                for (name, ty, e) in vars {
                    write!(f, " [{} {} {}]", name, ty, e)?;
                }
                write!(f, ")")
            },
            Stmt::Func { name, args, ret, body } => {
                write!(f, "(defn {} [", name)?;
                for (name, ty) in args {
                    write!(f, "[{} {}]", name, ty)?;
                }
                write!(f, "] {} {})", ret, body)
            },
        }
    }
}