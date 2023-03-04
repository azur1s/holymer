use std::fmt::{Display, Formatter, Result as FmtResult};
use crate::trans::ty::*;

#[derive(Clone, Debug)]
pub enum PUnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Debug)]
pub enum PBinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
}

#[derive(Clone, Debug)]
pub enum PLiteral { Num(i64), Str(String), Bool(bool) }

/// Enum to represent a parsed expression
#[derive(Clone, Debug)]
pub enum PExpr {
    Lit(PLiteral),
    Sym(String),

    Vec(Vec<Self>),

    UnaryOp(PUnaryOp, Box<Self>),
    BinaryOp(PBinaryOp, Box<Self>, Box<Self>),

    Call(Box<Self>, Vec<Self>),
    Lambda {
        args: Vec<(String, Type)>,
        body: Box<Self>,
    },
    Let {
        vars: Vec<(String, Type, Self)>,
        body: Box<Self>,
    }
}