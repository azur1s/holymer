use typing::typed::TExpr;
use syntax::expr::{Lit as ExprLit, UnaryOp, BinaryOp};

use std::fmt::{self, Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug)]
pub enum Lit<'src> {
    Unit,
    Bool(bool),
    Int(i64),
    Str(&'src str),
}

impl Display for Lit<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Lit::Unit    => write!(f, "()"),
            Lit::Bool(b) => write!(f, "{}", b),
            Lit::Int(i)  => write!(f, "{}", i),
            Lit::Str(s)  => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr<'src> {
    Lit(Lit<'src>),
    // v0
    Var(&'src str),
    // f(v0, v1, ...)
    Call(Vec<Self>),
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Expr::Lit(l) => write!(f, "{}", l),
            Expr::Var(s) => write!(f, "{}", s),
            Expr::Call(v) => {
                write!(f, "(")?;
                for (i, e) in v.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
        }
    }
}

macro_rules! unbox {
    ($e:expr) => {
        *(($e).0)
    };
}

macro_rules! str {
    ($e:expr) => {
        Expr::Lit(Lit::Str($e))
    };
}

macro_rules! var {
    ($e:expr) => {
        Expr::Var($e)
    };
}

macro_rules! call {
    ($e:expr) => {
        Expr::Call($e)
    };
}

pub fn lower_lit(lit: ExprLit) -> Lit {
    match lit {
        ExprLit::Unit    => Lit::Unit,
        ExprLit::Bool(b) => Lit::Bool(b),
        ExprLit::Int(i)  => Lit::Int(i),
        ExprLit::Str(s)  => Lit::Str(s),
    }
}

pub fn lower_expr(e: TExpr) -> Expr {
    match e {
        TExpr::Lit(l)   => Expr::Lit(lower_lit(l)),
        TExpr::Ident(s) => var!(s),
        TExpr::Unary { op, expr, .. } => {
            let expr = lower_expr(unbox!(expr));
            match op {
                UnaryOp::Neg => call!(vec![var!("neg"), expr]),
                UnaryOp::Not => call!(vec![var!("not"), expr]),
            }
        }
        TExpr::Binary { op: BinaryOp::Pipe, lhs, rhs, .. } => {
            let lhs = lower_expr(unbox!(lhs)); // arguments
            let rhs = lower_expr(unbox!(rhs)); // function
            call!(vec![rhs, lhs])
        }
        TExpr::Binary { op, lhs, rhs, .. } => {
            let lhs = lower_expr(unbox!(lhs));
            let rhs = lower_expr(unbox!(rhs));
            match op {
                BinaryOp::Add => call!(vec![var!("+"), lhs, rhs]),
                BinaryOp::Sub => call!(vec![var!("-"), lhs, rhs]),
                BinaryOp::Mul => call!(vec![var!("*"), lhs, rhs]),
                BinaryOp::Div => call!(vec![var!("/"), lhs, rhs]),
                BinaryOp::Rem => call!(vec![var!("%"), lhs, rhs]),
                BinaryOp::Eq  => call!(vec![var!("=="), lhs, rhs]),
                BinaryOp::Ne  => call!(vec![var!("!="), lhs, rhs]),
                BinaryOp::Lt  => call!(vec![var!("<"), lhs, rhs]),
                BinaryOp::Le  => call!(vec![var!("<="), lhs, rhs]),
                BinaryOp::Gt  => call!(vec![var!(">"), lhs, rhs]),
                BinaryOp::Ge  => call!(vec![var!(">="), lhs, rhs]),
                BinaryOp::And => call!(vec![var!("&&"), lhs, rhs]),
                BinaryOp::Or  => call!(vec![var!("||"), lhs, rhs]),
                BinaryOp::Pipe => unreachable!("pipe operator is handled separately"),
            }
        }
        TExpr::Lambda { params, body, .. } => {
            let body = lower_expr(unbox!(body));
            call!(vec![
                var!("lambda"),
                call!(params.into_iter().map(|(p, _)| var!(p)).collect()),
                body,
            ])
        }
        TExpr::Call { func, args } => {
            let func = lower_expr(unbox!(func));
            let args = args.into_iter()
                .map(|(a, _)| lower_expr(a))
                .collect::<Vec<_>>();
            call!(vec![func].into_iter().chain(args).collect())
        }
        TExpr::If { cond, t, f, br_ty } => todo!(),
        TExpr::Let { name, value, body, .. } => {
            let value = lower_expr(unbox!(value));
            let body = lower_expr(unbox!(body));
            call!(vec![var!("let"), str!(name), value, body])
        }
        TExpr::Define { name, value, .. } => {
            let value = lower_expr(unbox!(value));
            call!(vec![var!("define"), str!(name), value])
        }
        TExpr::Block { exprs, void, ret_ty } => todo!(),
    }
}