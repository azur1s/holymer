#![allow(clippy::new_without_default)]
use parser::{Expr, Literal, Span, Stmt};
use vm::model::{Instr, Value};

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile_expr(&mut self, expr: Expr) -> Vec<Instr> {
        match expr {
            Expr::Error => unreachable!(),
            Expr::Literal(x) => match x {
                Literal::Num(x) => vec![Instr::NumPush(x)],
                Literal::Bool(x) => vec![Instr::BoolPush(x)],
                Literal::Str(x) => vec![Instr::StrPush(x)],
            },
            Expr::Sym(name) => vec![Instr::Get(name)],
            Expr::Vec(xs) => {
                let mut instrs = vec![];
                let count = xs.len();
                for x in xs {
                    instrs.extend(self.compile_expr(x.0));
                }
                instrs.push(Instr::ListMake(count));
                instrs
            }
            Expr::Unary(op, x) => {
                let mut instrs = self.compile_expr(x.0);
                instrs.extend(match op.0 {
                    parser::UnaryOp::Neg => vec![Instr::NumPush(-1), Instr::NumMul],
                    parser::UnaryOp::Not => vec![Instr::BoolNot],
                });
                instrs
            }
            Expr::Binary(op, x, y) => {
                let mut instrs = self.compile_expr(y.0);
                instrs.extend(self.compile_expr(x.0));
                instrs.push(match op.0 {
                    parser::BinaryOp::Add => Instr::NumAdd,
                    parser::BinaryOp::Sub => Instr::NumSub,
                    parser::BinaryOp::Mul => Instr::NumMul,
                    parser::BinaryOp::Div => Instr::NumDiv,
                    // parser::BinaryOp::Eq => Instr::Eq,
                    // parser::BinaryOp::Ne => Instr::Neq,
                    // parser::BinaryOp::Lt => Instr::Lt,
                    // parser::BinaryOp::Gt => Instr::Gt,
                    // parser::BinaryOp::Le => Instr::Lte,
                    // parser::BinaryOp::Ge => Instr::Gte,
                    parser::BinaryOp::And => Instr::BoolAnd,
                    parser::BinaryOp::Or => Instr::BoolOr,
                    _ => todo!(),
                });
                instrs
            }
            Expr::Lambda(args, body) => {
                vec![Instr::FuncMake(args, self.compile_expr(body.0))]
            }
            Expr::Call(f, xs) => {
                let mut instrs = vec![];
                for x in xs {
                    instrs.extend(self.compile_expr(x.0));
                }
                if f.0 == Expr::Sym("print".to_string()) {
                    instrs.push(Instr::Print);
                } else {
                    instrs.extend(self.compile_expr(f.0));
                    instrs.push(Instr::FuncApply);
                }
                instrs
            }
            Expr::Let(binds, body) => {
                let mut instrs = vec![];
                let binds = binds
                    .into_iter()
                    .flat_map(|(name, expr)| {
                        let mut instrs = self.compile_expr(expr.0);
                        instrs.extend(vec![Instr::Set(name)]);
                        instrs
                    })
                    .collect::<Vec<_>>();
                if let Some(e) = body {
                    // If there is a body then we put the bindings
                    // inside the closure so it gets undefined outside
                    // the scope
                    instrs.extend(vec![
                        Instr::FuncMake(
                            vec![],
                            binds.into_iter().chain(self.compile_expr(e.0)).collect(),
                        ),
                        Instr::FuncApply,
                    ]);
                } else {
                    // If there is no body then we just push the bindings
                    // to the global scope
                    instrs.extend(binds);
                }
                instrs
            }
            Expr::If(c, t, f) => {
                let mut instrs = self.compile_expr(c.0);
                let t = self.compile_expr(t.0);
                if let Some(f) = f {
                    let f = self.compile_expr(f.0);
                    instrs.push(Instr::JumpIfFalse(t.len() + 1));
                    instrs.extend(t);
                    instrs.push(Instr::Jump(f.len()));
                    instrs.extend(f);
                } else {
                    instrs.push(Instr::JumpIfFalse(t.len()));
                    instrs.extend(t);
                }
                instrs
            }
            Expr::Do(es) => {
                let mut instrs = vec![];
                for e in es {
                    instrs.extend(self.compile_expr(e.0));
                }
                instrs
            }
        }
    }

    pub fn compile_stmt(&mut self, stmt: Stmt) -> Vec<Instr> {
        match stmt {
            Stmt::Fun(name, args, body) => {
                let is_main = name == "main";
                let mut instrs = vec![
                    Instr::FuncMake(args, self.compile_expr(body.0)),
                    Instr::Set(name),
                ];
                if is_main {
                    instrs.pop();
                    instrs.push(Instr::FuncApply);
                }
                instrs
            }
        }
    }

    pub fn compile_program(&mut self, stmts: Vec<(Stmt, Span)>) -> Vec<Instr> {
        let mut instrs = vec![];
        for (stmt, _) in stmts {
            instrs.extend(self.compile_stmt(stmt));
        }
        instrs
    }
}
