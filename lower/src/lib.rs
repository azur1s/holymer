#![allow(clippy::new_without_default)]
pub mod model;
use crate::model::{BinaryOp, Expr, Stmt};

pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Self {}
    }

    pub fn opt_stmts(&self, ss: Vec<Stmt>) -> Vec<Stmt> {
        ss.into_iter()
            .map(|s| self.opt_stmt(s.clone()).unwrap_or(s))
            .collect()
    }

    pub fn opt_stmt(&self, s: Stmt) -> Option<Stmt> {
        match s {
            Stmt::Fun(name, args, body) => Some(Stmt::Fun(
                name,
                args,
                self.opt_expr(body.clone()).unwrap_or(body),
            )),
            _ => None,
        }
    }

    pub fn opt_exprs(&self, es: Vec<Expr>) -> Vec<Expr> {
        es.into_iter()
            .map(|e| self.opt_expr(e.clone()).unwrap_or(e))
            .collect()
    }

    pub fn opt_expr(&self, e: Expr) -> Option<Expr> {
        match e {
            Expr::Binary(BinaryOp::Pipe, left, right) => Some(self.fold_pipe(*left, *right)),
            Expr::Lambda(args, body) => Some(Expr::Lambda(
                args,
                Box::new(self.opt_expr(*body.clone()).unwrap_or(*body)),
            )),
            Expr::Do(es) => Some(Expr::Do(self.opt_exprs(es))),
            _ => None,
        }
    }

    fn fold_pipe(&self, left: Expr, right: Expr) -> Expr {
        Expr::Call(
            Box::new(self.opt_expr(right.clone()).unwrap_or(right)),
            vec![self.opt_expr(left.clone()).unwrap_or(left)],
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::convert_expr;
    use parser::{lex, parse_expr};

    #[test]
    fn test_fold_pipe() {
        let s = "1 |> \\x -> x + 1";
        println!("{}", s);
        let (ts, es) = lex(s.to_owned());

        assert!(es.is_empty());

        let (ex, es) = parse_expr(ts.unwrap(), s.chars().count());

        assert!(es.is_empty());

        let ex = ex.unwrap();
        let ex = convert_expr(ex);
        let l = Lower::new();
        let ex = l.opt_expr(ex).unwrap();
        println!("{:?}", ex);
    }
}
