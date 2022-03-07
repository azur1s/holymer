use std::fmt::Display;

use hir::{IR, IRKind, Value};

const MODULE_INCLUDES: [&str; 3] = [
    "<stdbool.h>",
    "<iostream>",
    "<string>",
];

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
        for module in MODULE_INCLUDES {
            self.emit(format!("#include {}\n", module));
        }
        for ir in irs {
            self.emit(&self.gen_ir(&ir.kind));
        }
    }
    
    fn gen_ir(&self, ir: &IRKind) -> String {
        match ir {
            IRKind::Define { name, type_hint, value } => {
                format!("{} {} = {};\n", type_hint, name, self.gen_ir(value))
            },
            IRKind::Call { name, args } => {
                match name.as_str() {
                    "write" => { format!("std::cout << {};\n", self.gen_ir(&args[0])) },
                    "read" => { format!("std::cin >> {};\n", self.gen_ir(&args[0])) },
                    _ => format!("{}({});\n", name, args.iter().map(|arg| self.gen_ir(arg)).collect::<Vec<_>>().join(", ")),
                }
            },
            IRKind::Fun { name, return_type_hint, args, body } => {
                let args = args.iter().map(|arg| format!("{} {}", arg.1, arg.0)).collect::<Vec<_>>().join(", ");
                format!("{} {}({}) {{\n{}}}\n", return_type_hint, name, args, self.gen_ir(body))
            },
            IRKind::Return { value } => {
                format!("return {};\n", self.gen_ir(value))
            },
            IRKind::Do { body } => {
                let mut out = String::new();
                for expr in body {
                    out.push_str(&self.gen_ir(&expr));
                }
                out
            }

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