use crate::read::past::{PExpr, PLiteral, PBinaryOp, PUnaryOp};
use super::{
    ast::{Expr, Literal, BinaryOp, UnaryOp},
    js::{JSExpr, JSLiteral},
};

pub fn translate_expr(expr: PExpr) -> Expr {
    match expr {
        PExpr::Error => panic!("Error in expression!"),

        PExpr::Lit(l) => Expr::Lit(match l {
            PLiteral::Num(n)  => Literal::Num(n),
            PLiteral::Str(s)  => Literal::Str(s),
            PLiteral::Bool(b) => Literal::Bool(b),
        }),
        PExpr::Sym(s) => Expr::Sym(s),
        PExpr::Vec(v) => Expr::Vec(v.into_iter().map(|e| translate_expr(e.0)).collect()),

        PExpr::Unary(op, e) => Expr::UnaryOp(match op.0 {
            PUnaryOp::Neg => UnaryOp::Neg,
            PUnaryOp::Not => UnaryOp::Not,
        }, Box::new(translate_expr((*e).0))),
        PExpr::Binary((op, _), e1, e2) => Expr::BinaryOp(
            match op {
                PBinaryOp::Add => BinaryOp::Add,
                PBinaryOp::Sub => BinaryOp::Sub,
                PBinaryOp::Mul => BinaryOp::Mul,
                PBinaryOp::Div => BinaryOp::Div,
                PBinaryOp::Mod => BinaryOp::Mod,

                PBinaryOp::Eq  => BinaryOp::Eq,
                PBinaryOp::Neq => BinaryOp::Neq,

                PBinaryOp::Lt  => BinaryOp::Lt,
                PBinaryOp::Gt  => BinaryOp::Gt,
                PBinaryOp::Lte => BinaryOp::Lte,
                PBinaryOp::Gte => BinaryOp::Gte,

                PBinaryOp::And => BinaryOp::And,
                PBinaryOp::Or  => BinaryOp::Or,
            },
            Box::new(translate_expr((*e1).0)),
            Box::new(translate_expr((*e2).0)),
        ),

        PExpr::Call(f, args) => Expr::Call(
            Box::new(translate_expr((*f).0)),
            args.into_iter().map(|a| translate_expr(a.0)).collect(),
        ),
        PExpr::Lambda { args, body } => Expr::Lambda {
            args,
            body: Box::new(translate_expr((*body).0)),
        },
        PExpr::Let { vars, body } => {
            let mut expr: Expr = translate_expr(*body); // The expression we're building up
            for (name, ty, val) in vars.into_iter().rev() { // Reverse so we can build up the lambda
                // e.g.: let x : t = e1 in e2 end => (lambda (x : t) = e2)(e1)

                // Build up the lambda
                expr = Expr::Lambda {
                    args: vec![(name, ty)],
                    body: Box::new(expr),
                };
                // Call the lambda with the value
                let val = translate_expr(val);
                expr = Expr::Call(Box::new(expr), vec![val]);
            }

            expr
        }
    }
}

pub fn translate_js(expr: Expr) -> JSExpr {
    match expr {
        Expr::Lit(l) => match l {
            Literal::Num(n)  => JSExpr::Lit(JSLiteral::Num(n)),
            Literal::Str(s)  => JSExpr::Lit(JSLiteral::Str(s)),
            Literal::Bool(b) => JSExpr::Lit(JSLiteral::Bool(b)),
        },
        Expr::Sym(s) => JSExpr::Sym(s),
        Expr::Vec(v) => JSExpr::Array(v.into_iter().map(translate_js).collect()),

        Expr::UnaryOp(op, e) => JSExpr::Op(match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }, Box::new(translate_js(*e)), None),
        Expr::BinaryOp(op, e1, e2) => JSExpr::Op(match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",

            BinaryOp::Eq  => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::Lt  => "<",
            BinaryOp::Gt  => ">",
            BinaryOp::Lte => "<=",
            BinaryOp::Gte => ">=",

            BinaryOp::And => "&&",
            BinaryOp::Or  => "||",
        }, Box::new(translate_js(*e1)), Some(Box::new(translate_js(*e2)))),

        Expr::Call(f, args) => {
            match *f {
                Expr::Sym(ref s) => {
                    match s.as_str() {
                        "println" => {
                            JSExpr::Method(
                                Box::new(JSExpr::Sym("console".to_string())),
                                "log".to_string(),
                                args.into_iter().map(translate_js).collect(),
                            )
                        },
                        _ => JSExpr::Call(
                            Box::new(translate_js(*f)),
                            args.into_iter().map(translate_js).collect(),
                        ),
                    }
                },
                _ => JSExpr::Call(
                    Box::new(translate_js(*f)),
                    args.into_iter().map(translate_js).collect(),
                ),
            }
        }
        Expr::Lambda { args, body } => JSExpr::Lambda {
            args,
            body: Box::new(translate_js(*body)),
        },
    }
}