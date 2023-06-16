use chumsky::span::SimpleSpan;
use syntax::expr::{Lit, UnaryOp, BinaryOp};
use typing::typed::TExpr;

#[derive(Clone, Debug)]
pub enum IExpr<'src> {
    BoolAnd,
    IntPush(i64),
    IntAdd,
    IntSub,
    IntRem,
    IntEq,
    StrPush(&'src str),
    Branch(Vec<Self>, Vec<Self>),
    VarLoad(&'src str),
    VarStore(&'src str),
    FnPush(Vec<Self>),
    Call,
    Ret,
}

#[derive(Clone, Debug)]
pub struct Lowerer {
}

impl Lowerer {
    pub fn new() -> Self {
        Self {}
    }

    fn lower_texpr<'a>(self: &mut Self, e: TExpr<'a>) -> Vec<IExpr<'a>> {
        use IExpr::*;
        match e {
            TExpr::Lit(l) => match l {
                Lit::Unit    => vec![],
                Lit::Bool(_) => todo!(),
                Lit::Int(n)  => vec![IntPush(n)],
                Lit::Str(s)  => vec![StrPush(s)],
            }
            TExpr::Ident(s) => vec![VarLoad(s)],
            TExpr::Unary { op, expr, .. } => {
                let mut expr = self.lower_texpr(*expr.0);
                expr.push(match op {
                    UnaryOp::Neg => IntSub,
                    UnaryOp::Not => todo!(),
                });
                expr
            }
            TExpr::Binary { op, lhs, rhs, .. } if op == BinaryOp::Pipe => {
                println!("{lhs:?}");
                println!("{rhs:?}");
                todo!()
            }
            TExpr::Binary { op, lhs, rhs, .. } => {
                let mut lhs = self.lower_texpr(*lhs.0);
                let mut rhs = self.lower_texpr(*rhs.0);
                lhs.append(&mut rhs);
                lhs.push(match op {
                    BinaryOp::Add => IExpr::IntAdd,
                    BinaryOp::Sub => IExpr::IntSub,
                    BinaryOp::Mul => todo!(),
                    BinaryOp::Div => todo!(),
                    BinaryOp::Rem => IExpr::IntRem,
                    BinaryOp::Eq  => IExpr::IntEq,
                    BinaryOp::Ne  => todo!(),
                    BinaryOp::Lt  => todo!(),
                    BinaryOp::Gt  => todo!(),
                    BinaryOp::Le  => todo!(),
                    BinaryOp::Ge  => todo!(),
                    BinaryOp::And => IExpr::BoolAnd,
                    BinaryOp::Or  => todo!(),
                    BinaryOp::Pipe => unreachable!(),
                });
                lhs
            }

            TExpr::Lambda { body, .. } => {
                let mut es = self.lower_texpr(*body.0);
                es.push(IExpr::Ret);
                vec![IExpr::FnPush(es)]
            },
            TExpr::Call { func, args } => {
                let mut es: Vec<IExpr> = args.into_iter()
                    .flat_map(|(e, _)| self.lower_texpr(e))
                    .collect();
                es.append(&mut self.lower_texpr(*func.0));
                es.push(IExpr::Call);
                es
            },

            TExpr::If { cond, t, f, .. } => {
                let mut es = self.lower_texpr(*cond.0);
                es.push(IExpr::Branch(
                    self.lower_texpr(*t.0),
                    self.lower_texpr(*f.0),
                ));
                es
            },
            TExpr::Define { name, value, .. } => {
                let mut es = self.lower_texpr(*value.0);
                es.push(IExpr::VarStore(name));
                es
            },


            e => unimplemented!("{:?}", e)
        }
    }

    pub fn lower_texprs<'a>(self: &mut Self, e: Vec<(TExpr<'a>, SimpleSpan)>) -> Vec<IExpr<'a>> {
        e.into_iter()
            .flat_map(|(e, _)| self.lower_texpr(e))
            .collect()
    }
}