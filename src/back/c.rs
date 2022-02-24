use crate::front::parse::Expr;

pub struct Codegen {
    pub emitted: String,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            emitted: String::new(),
        }
    }

    fn emit(&mut self, s: String) {
        self.emitted.push_str(s.as_str());
    }

    pub fn gen(&mut self, exprs: &[Expr]) {
        self.emit("#include <stdio.h>\n#include <stdbool.h>\nint main() {\n".to_string());
        for expr in exprs {
            self.gen_expr(expr);
        }
        self.emit("return 0;\n}\n".to_string());
    }
    
    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Let { name, value } => {
                match &**value {
                    Expr::Int(i)     => self.emit(format!("int {} = {};\n", name, i)),
                    Expr::Float(f)   => self.emit(format!("double {} = {};\n", name, f)),
                    Expr::Boolean(b) => self.emit(format!("bool {} = {};\n", name, b)),
                    Expr::String(s)  => self.emit(format!("char *{} = \"{}\";\n", name, s)),
                    _ => todo!(),
                }
            },
            Expr::Call { name, args } => {
                match &**name {
                    Expr::Ident(func) => {
                        match func.as_str() {
                            "print" => {
                                self.emit(format!("printf({});\n", match &args[0] {
                                    Expr::String(s) => format!("\"{}\"", s),
                                    Expr::Ident(s) => s.to_string(),
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
                self.emit("if ".to_string());
                self.gen_expr(&cond);
                self.emit(" {\n".to_string());
                self.gen_expr(&then);
                self.emit("} else {\n".to_string());
                self.gen_expr(&else_);
                self.emit("}\n".to_string());
            },
            Expr::Binary { left, op, right } => {
                self.emit("(".to_string());
                self.gen_expr(&left);
                self.emit(format!(" {} ", op.to_string()));
                self.gen_expr(&right);
                self.emit(")".to_string());
            },
            Expr::Ident(s) => self.emit(s.to_string()),
            Expr::Boolean(b) => self.emit(format!("{}", b)),
            _ => { println!("{:?}", expr); todo!() },
        }
    }

}