use std::fmt::Display;

use hir::{IR, IRKind, Value};

pub struct Codegen {
    pub emitted: String,
}

impl Codegen {
    pub fn new() -> Self {
        Self { emitted: String::new() }
    }

    fn emit<T: Display>(&mut self, t: T) {
        self.emitted.push_str(&t.to_string());
    }

    pub fn gen(&mut self, irs: Vec<IR>) {
        self.emit("#include <stdbool.h>\n");
        self.emit("#include <string>\n");
        self.emit("int main() {\n");
        for ir in irs {
            self.emit(&self.gen_ir(&ir.kind));
        }
        self.emit("}");
    }
    
    fn gen_ir(&self, ir: &IRKind) -> String {
        match ir {
            IRKind::Define { name, type_hint, value } => {
                format!("{} {} = {};\n", type_hint, name, self.gen_ir(value))
            },
            IRKind::Value { value } => {
                match value {
                    Value::Int(value)     => format!("{}", value),
                    Value::Boolean(value) => format!("{}", value),
                    Value::String(value)  => format!("\"{}\"", value),
                    Value::Ident(value)   => format!("{}", value),
                }
            },
            _ => { dbg!(ir); todo!() },
        }
    }
}