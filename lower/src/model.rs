use crate::model;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Num(i64),
    Bool(bool),
    Str(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Pipe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Error,

    Literal(Literal),
    Sym(String),
    Vec(Vec<Self>),
    Unary(UnaryOp, Box<Self>),
    Binary(BinaryOp, Box<Self>, Box<Self>),
    Lambda(Vec<String>, Box<Self>),
    Call(Box<Self>, Vec<Self>),
    Let(Vec<(String, Self)>, Option<Box<Self>>),
    If(Box<Self>, Box<Self>, Option<Box<Self>>),
    Do(Vec<Expr>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Fun(String, Vec<String>, Expr),
}

pub fn converts(s: Vec<(parser::Stmt, std::ops::Range<usize>)>) -> Vec<model::Stmt> {
    s.into_iter().map(|(s, _)| convert(s)).collect()
}

pub fn convert(s: parser::Stmt) -> model::Stmt {
    match s {
        parser::Stmt::Fun(name, args, body) => model::Stmt::Fun(name, args, convert_expr(body)),
    }
}

pub fn convert_expr(e: (parser::Expr, std::ops::Range<usize>)) -> model::Expr {
    match e.0 {
        parser::Expr::Error => model::Expr::Error,

        parser::Expr::Literal(l) => match l {
            parser::Literal::Num(n) => model::Expr::Literal(model::Literal::Num(n)),
            parser::Literal::Bool(b) => model::Expr::Literal(model::Literal::Bool(b)),
            parser::Literal::Str(s) => model::Expr::Literal(model::Literal::Str(s)),
        },
        parser::Expr::Sym(s) => model::Expr::Sym(s),
        parser::Expr::Vec(es) => model::Expr::Vec(es.into_iter().map(convert_expr).collect()),
        parser::Expr::Unary(op, e) => {
            model::Expr::Unary(convert_unary_op(op.0), Box::new(convert_expr(*e)))
        }
        parser::Expr::Binary(op, left, right) => model::Expr::Binary(
            convert_binary_op(op.0),
            Box::new(convert_expr(*left)),
            Box::new(convert_expr(*right)),
        ),
        parser::Expr::Lambda(args, body) => {
            model::Expr::Lambda(args, Box::new(convert_expr(*body)))
        }
        parser::Expr::Call(f, args) => model::Expr::Call(
            Box::new(convert_expr(*f)),
            args.into_iter().map(convert_expr).collect(),
        ),
        parser::Expr::Let(bindings, body) => model::Expr::Let(
            bindings
                .into_iter()
                .map(|(s, e)| (s, convert_expr(e)))
                .collect(),
            body.map(|e| Box::new(convert_expr(*e))),
        ),
        parser::Expr::If(cond, then, else_) => model::Expr::If(
            Box::new(convert_expr(*cond)),
            Box::new(convert_expr(*then)),
            else_.map(|e| Box::new(convert_expr(*e))),
        ),
        parser::Expr::Do(es) => model::Expr::Do(es.into_iter().map(convert_expr).collect()),
    }
}

pub fn convert_unary_op(op: parser::UnaryOp) -> model::UnaryOp {
    match op {
        parser::UnaryOp::Neg => model::UnaryOp::Neg,
        parser::UnaryOp::Not => model::UnaryOp::Not,
    }
}

pub fn convert_binary_op(op: parser::BinaryOp) -> model::BinaryOp {
    match op {
        parser::BinaryOp::Add => model::BinaryOp::Add,
        parser::BinaryOp::Sub => model::BinaryOp::Sub,
        parser::BinaryOp::Mul => model::BinaryOp::Mul,
        parser::BinaryOp::Div => model::BinaryOp::Div,
        parser::BinaryOp::Lt => model::BinaryOp::Lt,
        parser::BinaryOp::Le => model::BinaryOp::Le,
        parser::BinaryOp::Gt => model::BinaryOp::Gt,
        parser::BinaryOp::Ge => model::BinaryOp::Ge,
        parser::BinaryOp::Eq => model::BinaryOp::Eq,
        parser::BinaryOp::Ne => model::BinaryOp::Ne,
        parser::BinaryOp::And => model::BinaryOp::And,
        parser::BinaryOp::Or => model::BinaryOp::Or,
        parser::BinaryOp::Pipe => model::BinaryOp::Pipe,
    }
}
