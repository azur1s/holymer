use chumsky::span::SimpleSpan;
use syntax::ty::Type;

use crate::typed::TExpr;

/// A renamer to rename type variables to a "minimized" form for more readable output
pub struct Renamer {
    // Type variables encountered so far
    vars: Vec<usize>,
}

impl<'src> Renamer {
    pub fn new() -> Self {
        Self {
            vars: vec![],
        }
    }

    fn rename_var(&self, i: usize) -> Type {
        let n = self.vars.iter().position(|x| x == &i).unwrap();
        Type::Var(n)
    }

    fn add_var(&mut self, i: usize) {
        if !self.vars.contains(&i) {
            self.vars.push(i);
        }
    }

    fn find_var(&mut self, t: Type) {
        match t {
            Type::Var(i) => {
                self.add_var(i);
            },
            Type::Func(args, ret) => {
                args.into_iter().for_each(|t| self.find_var(t));
                self.find_var(*ret);
            },
            Type::Tuple(tys) => {
                tys.into_iter().for_each(|t| self.find_var(t));
            },
            Type::Array(ty) => {
                self.find_var(*ty);
            },
            _ => {},
        }
    }

    fn traverse(&mut self, e: TExpr) {
        match e {
            TExpr::Unary { expr, ret_ty, ..} => {
                self.traverse(*expr.0);
                self.find_var(ret_ty);
            },
            TExpr::Binary { lhs, rhs, ret_ty, ..} => {
                self.traverse(*lhs.0);
                self.traverse(*rhs.0);
                self.find_var(ret_ty);
            },
            TExpr::Lambda { params, body, ret_ty } => {
                for (_, t) in params { self.find_var(t); }
                self.find_var(ret_ty);
                self.traverse(*body.0);
            },
            TExpr::Call { func, args } => {
                self.traverse(*func.0);
                for arg in args {
                    self.traverse(arg.0);
                }
            },
            TExpr::Let { ty, value, body, .. } => {
                self.find_var(ty);
                self.traverse(*value.0);
                self.traverse(*body.0);
            },
            TExpr::Define { ty, value, .. } => {
                self.find_var(ty);
                self.traverse(*value.0);
            },
            TExpr::Block { exprs, ret_ty, .. } => {
                for expr in exprs {
                    self.traverse(expr.0);
                }
                self.find_var(ret_ty);
            },
            _ => {},
        }
    }

    fn rename_type(&self, t: Type) -> Type {
        match t {
            Type::Var(i) => self.rename_var(i),
            Type::Func(args, ret) => {
                Type::Func(
                    args.into_iter().map(|x| self.rename_type(x)).collect(),
                    Box::new(self.rename_type(*ret)),
                )
            },
            Type::Tuple(tys) => {
                Type::Tuple(tys.into_iter().map(|x| self.rename_type(x)).collect())
            },
            Type::Array(ty) => {
                Type::Array(Box::new(self.rename_type(*ty)))
            },
            _ => t,
        }
    }

    fn rename_texp(&self, e: TExpr<'src>) -> TExpr<'src> {
        match e {
            TExpr::Unary { op, expr, ret_ty } => {
                TExpr::Unary {
                    op,
                    expr: (Box::new(self.rename_texp(*expr.0)), expr.1),
                    ret_ty: self.rename_type(ret_ty)
                }
            },
            TExpr::Binary { op, lhs, rhs, ret_ty } => {
                TExpr::Binary {
                    op,
                    lhs: (Box::new(self.rename_texp(*lhs.0)), lhs.1),
                    rhs: (Box::new(self.rename_texp(*rhs.0)), rhs.1),
                    ret_ty: self.rename_type(ret_ty)
                }
            },
            TExpr::Lambda { params, body, ret_ty } => {
                TExpr::Lambda {
                    params: params.into_iter()
                        .map(|(x, t)| (x, self.rename_type(t)))
                        .collect(),
                    body: (Box::new(self.rename_texp(*body.0)), body.1),
                    ret_ty: self.rename_type(ret_ty)
                }
            },
            TExpr::Call { func, args } => {
                TExpr::Call {
                    func: (Box::new(self.rename_texp(*func.0)), func.1),
                    args: args.into_iter()
                        .map(|x| (self.rename_texp(x.0), x.1))
                        .collect()
                }
            },
            TExpr::Let { name, ty, value, body } => {
                TExpr::Let {
                    name,
                    ty: self.rename_type(ty),
                    value: (Box::new(self.rename_texp(*value.0)), value.1),
                    body: (Box::new(self.rename_texp(*body.0)), body.1)
                }
            },
            TExpr::Define { name, ty, value } => {
                TExpr::Define {
                    name,
                    ty: self.rename_type(ty),
                    value: (Box::new(self.rename_texp(*value.0)), value.1)
                }
            },
            TExpr::Block { exprs, void, ret_ty } => {
                TExpr::Block {
                    exprs: exprs.into_iter()
                        .map(|x| (self.rename_texp(x.0), x.1))
                        .collect(),
                    void,
                    ret_ty: self.rename_type(ret_ty)
                }
            },
            _ => e,
        }
    }
}

pub fn rename_type(t: Type) -> Type {
    let mut renamer = Renamer::new();
    renamer.find_var(t.clone());
    renamer.rename_type(t)
}

pub fn rename_exprs(es: Vec<(TExpr, SimpleSpan)>) -> Vec<(TExpr, SimpleSpan)> {
    let mut renamer = Renamer::new();
    es.clone().into_iter()
        .for_each(|e| renamer.traverse(e.0));
    es.into_iter()
        .map(|(e, s)| (renamer.rename_texp(e), s))
        .collect()
}