use crate::trans::ty::*;

use crate::read::parse::Spanned;

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
pub enum PLiteral { Num(i64), Str(String), Bool(bool), Unit }

#[derive(Clone, Debug)]
pub enum PExpr {
    Error,

    Lit(PLiteral),
    Sym(String),
    Vec(Vec<Spanned<Self>>),

    Unary(Spanned<PUnaryOp>, Box<Spanned<Self>>),
    Binary(Spanned<PBinaryOp>, Box<Spanned<Self>>, Box<Spanned<Self>>),

    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    Lambda {
        args: Vec<(String, Type)>,
        body: Box<Spanned<Self>>,
    },
    Let {
        vars: Vec<(String, Type, Spanned<Self>)>,
        body: Option<Box<Spanned<Self>>>,
    },
    Block(Vec<Spanned<Self>>),
    Return(Box<Spanned<Self>>),
}

#[derive(Clone, Debug)]
pub enum PStmt {
    Expr(Spanned<PExpr>),
    Func {
        name: String,
        args: Vec<(String, Type)>,
        ret: Type,
        body: Box<Spanned<PExpr>>,
    },
}