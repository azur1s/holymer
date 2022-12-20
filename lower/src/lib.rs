#![allow(clippy::new_without_default)]
use parser::*;

type SExpr = (Expr, std::ops::Range<usize>);

pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Self {}
    }

    pub fn opt_stmts(&self, ss: Vec<Stmt>) -> Vec<Stmt> {
        ss.into_iter()
            .flat_map(|s| self.opt_stmt(s.clone()).unwrap_or_else(|| vec![s]))
            .collect()
    }

    pub fn opt_stmt(&self, s: Stmt) -> Option<Vec<Stmt>> {
        match s {
            // Stmt::Fun(name, args, body) => Some(vec![Stmt::Fun(
            //     name,
            //     args,
            //     self.opt_expr(body.0).unwrap_or_else(|| vec![body.0]),
            // )]),
            _ => None,
        }
    }

    pub fn opt_exprs(&self, es: Vec<Expr>) -> Vec<Expr> {
        es.into_iter()
            .flat_map(|e| self.opt_expr(e.clone()).unwrap_or_else(|| vec![e]))
            .collect()
    }

    pub fn opt_expr(&self, e: Expr) -> Option<Vec<Expr>> {
        match e {
            Expr::Binary((BinaryOp::Pipe, _), left, right) => Some(self.fold_pipe(*left, *right)),
            _ => None,
        }
    }

    fn fold_pipe(&self, left: SExpr, right: SExpr) -> Vec<Expr> {
        vec![Expr::Call(Box::new(right), vec![left])]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fold_pipe() {
        let s = "1 |> \\x -> x + 1";
        println!("{}", s);
        let (ts, es) = lex(s.to_owned());

        assert!(es.is_empty());

        let (ex, es) = parse_expr(ts.unwrap(), s.chars().count());

        assert!(es.is_empty());

        let ex = ex.unwrap();
        let l = Lower::new();
        let ex = l.opt_expr(ex.0).unwrap();
        println!("{:?}", ex);
    }
}
