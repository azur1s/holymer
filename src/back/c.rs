use std::fmt::Display;

use crate::front::parse::Expr;

pub struct Codegen {
    pub emitted: String,
}

const HEADER_INCLUDES: [&str; 2] = [
    "#include <stdio.h>",
    "#include <hycron/stdbool.h>",
];

impl Codegen {
    pub fn new() -> Self {
        Self {
            emitted: String::new(),
        }
    }

    fn emit<T: Display>(&mut self, s: T) {
        self.emitted.push_str(&s.to_string());
    }

    pub fn gen(&mut self, exprs: &[Expr]) {
        for header in HEADER_INCLUDES.iter() {
            self.emit(header);
        }
        self.emit("int main() {");
        for expr in exprs {
            self.gen_expr(expr);
        }
        self.emit("return 0;");
        self.emit("}");
    }
    
    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Let { name, value } => {
                match &**value {
                    Expr::Int(i)     => self.emit(format!("int {} = {};", name, i)),
                    Expr::Float(f)   => self.emit(format!("double {} = {};", name, f)),
                    Expr::Boolean(b) => self.emit(format!("bool {} = {};", name, b)),
                    Expr::String(s)  => self.emit(format!("char *{} = \"{}\";", name, s)),
                    _ => todo!(),
                }
            },
            Expr::Call { name, args } => {
                match &**name {
                    Expr::Ident(func) => {
                        match func.as_str() {
                            "print" => {
                                self.emit(format!("printf({});", match &args[0] {
                                    Expr::String(s) => format!("\"{}\"", s),
                                    Expr::Ident(s) => format!("\"%s\", {}", s),
                                    _ => todo!(),
                                }));
                            },
                            _ => todo!(),
                        }
                    },
                    _ => todo!(),
                }
            },
            Expr::If { cond, then, else_ } => {
                self.emit("if (".to_string());
                self.gen_expr(&cond);
                self.emit(") {".to_string());
                self.gen_expr(&then);
                self.emit("} else {".to_string());
                self.gen_expr(&else_);
                self.emit("}".to_string());
            },
            Expr::Binary { left, op, right } => {
                self.gen_expr(&left);
                self.emit(format!(" {} ", op.to_string()));
                self.gen_expr(&right);
            },
            Expr::Ident(s) => self.emit(s.to_string()),
            Expr::Boolean(b) => self.emit(format!("{}", b)),
            _ => { println!("{:?}", expr); todo!() },
        }
    }

}