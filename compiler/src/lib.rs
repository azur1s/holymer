#![allow(clippy::new_without_default)]
#![allow(clippy::only_used_in_recursion)]
use lower::model::{BinaryOp, Expr, Literal, Stmt, UnaryOp};
use vm::model::Instr;

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile_expr(&mut self, expr: Expr) -> Vec<Instr> {
        match expr {
            Expr::Error => {
                println!("{:?}", expr);
                unreachable!()
            }
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
                    instrs.extend(self.compile_expr(x));
                }
                instrs.push(Instr::ListMake(count));
                instrs
            }
            Expr::Unary(op, x) => {
                let mut instrs = self.compile_expr(*x);
                instrs.extend(match op {
                    UnaryOp::Neg => vec![Instr::NumPush(-1), Instr::NumMul],
                    UnaryOp::Not => vec![Instr::BoolNot],
                });
                instrs
            }
            Expr::Binary(op, x, y) => {
                let mut instrs = self.compile_expr(*y);
                instrs.extend(self.compile_expr(*x));
                instrs.push(match op {
                    BinaryOp::Add => Instr::NumAdd,
                    BinaryOp::Sub => Instr::NumSub,
                    BinaryOp::Mul => Instr::NumMul,
                    BinaryOp::Div => Instr::NumDiv,
                    BinaryOp::Eq => Instr::NumEq,
                    BinaryOp::Ne => Instr::NumNe,
                    BinaryOp::Lt => Instr::NumLt,
                    BinaryOp::Gt => Instr::NumGt,
                    BinaryOp::Le => Instr::NumLe,
                    BinaryOp::Ge => Instr::NumGe,
                    BinaryOp::And => Instr::BoolAnd,
                    BinaryOp::Or => Instr::BoolOr,
                    BinaryOp::Pipe => todo!(),
                });
                instrs
            }
            Expr::Lambda(args, body) => {
                vec![Instr::FuncMake(args, self.compile_expr(*body))]
            }
            Expr::Call(f, xs) => {
                let mut instrs = vec![];
                for x in xs {
                    instrs.extend(self.compile_expr(x));
                }
                match *f {
                    Expr::Sym(ref fname) => match fname.as_str() {
                        "print" => instrs.push(Instr::Print),
                        "println" => instrs.push(Instr::PrintLn),
                        _ => {
                            instrs.extend(self.compile_expr(*f));
                            instrs.push(Instr::FuncApply);
                        }
                    },
                    Expr::Lambda(_, _) => {
                        instrs.extend(self.compile_expr(*f));
                        instrs.push(Instr::FuncApply);
                    }
                    _ => todo!(),
                }
                instrs
            }
            Expr::Let(binds, body) => {
                let mut instrs = vec![];
                let binds = binds
                    .into_iter()
                    .flat_map(|(name, expr)| {
                        let mut instrs = self.compile_expr(expr);
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
                            binds.into_iter().chain(self.compile_expr(*e)).collect(),
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
                let mut instrs = self.compile_expr(*c);
                let t = self.compile_expr(*t);
                if let Some(f) = f {
                    let f = self.compile_expr(*f);
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
                    instrs.extend(self.compile_expr(e));
                }
                instrs
            }
        }
    }

    pub fn compile_stmt(&mut self, stmt: Stmt) -> Vec<Instr> {
        match stmt {
            Stmt::Fun(name, args, body) => {
                let is_main = name == "main";
                let mut instrs = match body {
                    // If the body is a lambda then we don't have to compile
                    // it into a function
                    Expr::Lambda(_, _) => self.compile_expr(body),
                    _ => vec![Instr::FuncMake(args, self.compile_expr(body))],
                };
                instrs.push(Instr::Set(name));
                if is_main {
                    instrs.pop();
                    instrs.push(Instr::FuncApply);
                }
                instrs
            }
        }
    }

    pub fn compile_program(&mut self, stmts: Vec<Stmt>) -> Vec<Instr> {
        let mut instrs = vec![];
        for stmt in stmts {
            instrs.extend(self.compile_stmt(stmt));
        }
        instrs
    }
}
