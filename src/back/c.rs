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

    fn emit_str(&mut self, s: &str) {
        self.emitted.push_str(s);
    }

    pub fn gen(&mut self, exprs: &[Expr]) {
        self.emit_str("#include <stdio.h>\n");
        self.emit_str("#include <hycron/bool.h>\n");
        self.emit_str("int main() {\n");
        for expr in exprs {
            self.gen_expr(expr);
        }
        self.emit_str("return 0;\n");
        self.emit_str("}\n");
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