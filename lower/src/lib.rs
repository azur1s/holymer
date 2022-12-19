#![allow(clippy::new_without_default)]
use parser::*;

pub struct Lower {}

impl Lower {
    pub fn new() -> Self {
        Self {}
    }

    fn fold_pipe(&self, e: Expr) -> Vec<Expr> {
        if let Expr::Binary((BinaryOp::Pipe, _), left, right) = e {
            vec![Expr::Call(right, vec![*left])]
        } else {
            unreachable!()
        }
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
        let ex = l.fold_pipe(ex.0);
        println!("{:?}", ex);
    }
}
