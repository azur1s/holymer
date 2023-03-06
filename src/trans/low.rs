use crate::asts::{
    past::*,
    ast::*,
    js::*,
};

pub fn translate_stmt(stmt: PStmt) -> Stmt {
    match stmt {
        PStmt::Expr(e) => Stmt::Expr(translate_expr(e.0)),
        PStmt::Func { name, args, ret, body } => Stmt::Func {
            name,
            args: args.into_iter().map(|(name, _ty)| name).collect(),
            ret,
            body: translate_expr(body.0),
        },
    }
}

pub fn exprs_to_lam(es: Vec<PExpr>) -> Expr {
    let lam = Expr::Lambda {
        args: vec![],
        body: es.into_iter().map(translate_expr).collect(),
    };
    Expr::Call(Box::new(lam), vec![])
}

pub fn translate_expr(expr: PExpr) -> Expr {
    match expr {
        PExpr::Error => panic!("Error in expression!"),

        PExpr::Lit(l) => Expr::Lit(match l {
            PLiteral::Num(n)  => Literal::Num(n),
            PLiteral::Str(s)  => Literal::Str(s),
            PLiteral::Bool(b) => Literal::Bool(b),
            PLiteral::Unit    => Literal::Unit,
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
            args: args.into_iter().map(|(name, _ty)| name).collect(),
            body: vec![translate_expr((*body).0)],
        },
        PExpr::Let { vars, body } => {
            if let Some(body) = body {
                let mut expr: Expr = translate_expr(body.0); // The expression we're building up
                for (name, _ty, val) in vars.into_iter().rev() { // Reverse so we can build up the lambda
                    // e.g.: let x : t = e1 in e2; => (lambda (x : t) = e2)(e1)
                    // Build up the lambda
                    expr = Expr::Lambda {
                        args: vec![name],
                        body: vec![expr],
                    };
                    // Call the lambda with the value
                    let val = translate_expr(val.0);
                    expr = Expr::Call(Box::new(expr), vec![val]);
                }
                expr
            } else {
                Expr::Defines(vars.into_iter().map(|(name, _ty, val)| {
                    (name, translate_expr(val.0))
                }).collect())
            }
        },
        PExpr::If { cond, t, f } => Expr::If {
            cond: Box::new(translate_expr((*cond).0)),
            t: Box::new(translate_expr((*t).0)),
            f: Box::new(translate_expr((*f).0)),
        },
        PExpr::Block(es) => {
            exprs_to_lam(es.into_iter().map(|e| e.0).collect())
        },
        PExpr::Return(e) => Expr::Return(Box::new(translate_expr((*e).0))),
    }
}

pub fn translate_js_stmt(stmt: Stmt) -> JSStmt {
    match stmt {
        Stmt::Expr(e) => JSStmt::Expr(translate_js_expr(e)),
        Stmt::Func { name, args, ret, body } => JSStmt::Func {
            name,
            args,
            ret,
            body: translate_js_expr(body),
        },
    }
}

pub fn translate_js_expr(expr: Expr) -> JSExpr {
    match expr {
        Expr::Lit(l) => match l {
            Literal::Num(n)  => JSExpr::Lit(JSLiteral::Num(n)),
            Literal::Str(s)  => JSExpr::Lit(JSLiteral::Str(s)),
            Literal::Bool(b) => JSExpr::Lit(JSLiteral::Bool(b)),
            Literal::Unit    => JSExpr::Lit(JSLiteral::Undefined),
        },
        Expr::Sym(s) => JSExpr::Sym(s),
        Expr::Vec(v) => JSExpr::Array(v.into_iter().map(translate_js_expr).collect()),

        Expr::UnaryOp(op, e) => JSExpr::Op(match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }, Box::new(translate_js_expr(*e)), None),
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
        }, Box::new(translate_js_expr(*e1)), Some(Box::new(translate_js_expr(*e2)))),

        Expr::Call(f, args) => {
            match *f {
                Expr::Sym(ref s) => {
                    match s.as_str() {
                        "println" => JSExpr::Call(
                                Box::new(JSExpr::Method(
                                    Box::new(JSExpr::Sym("console".to_string())),
                                    "log".to_string(),
                                )),
                                args.into_iter().map(translate_js_expr).collect()),
                        "print" => JSExpr::Call(
                                Box::new(JSExpr::Method(
                                    Box::new(JSExpr::Method(
                                        Box::new(JSExpr::Sym("process".to_string())),
                                        "stdout".to_string(),
                                    )),
                                    "write".to_string(),
                                )),
                                args.into_iter().map(translate_js_expr).collect()),
                        _ => JSExpr::Call(
                            Box::new(translate_js_expr(*f)),
                            args.into_iter().map(translate_js_expr).collect(),
                        ),
                    }
                },
                _ => JSExpr::Call(
                    Box::new(translate_js_expr(*f)),
                    args.into_iter().map(translate_js_expr).collect(),
                ),
            }
        }
        Expr::Lambda { args, body } => JSExpr::Lambda {
            args,
            body: body.into_iter().map(translate_js_expr).collect(),
        },
        Expr::If { cond, t, f } => JSExpr::If {
            cond: Box::new(translate_js_expr(*cond)),
            t: Box::new(translate_js_expr(*t)),
            f: Box::new(translate_js_expr(*f)),
        },
        Expr::Defines(defs) => JSExpr::Defines(defs.into_iter().map(|(name, val)| {
            (name, translate_js_expr(val))
        }).collect()),
        Expr::Return(e) => JSExpr::Return(Box::new(translate_js_expr(*e))),
    }
}