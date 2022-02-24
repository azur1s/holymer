use std::fmt::Display;

use crate::middle::ir::{IR, Value};

#[derive(Debug, Clone)]
pub struct Codegen {
    pub emitted: String,
}

const HEADER_INCLUDES: [&str; 3] = [
    "<stdio.h>",
    "<string.h>",
    "<hycron/stdbool.h>",
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

    pub fn gen(&mut self, irs: &[IR]) {
        for header in HEADER_INCLUDES.iter() {
            self.emit("#include ");
            self.emit(header);
            self.emit("\n");
        }
        for ir in irs {
            self.gen_ir(ir);
        }
    }

    fn gen_ir(&mut self, ir: &IR) {
        match ir {
            IR::Define { name, type_hint, value } => {
                self.emit(format!("{} {} = ", type_hint, name));
                self.gen_ir(value);
                self.emit(";\n");
            },
            IR::Fun { name, return_type_hint, args, body } => {
                let args = args.iter().map(|(name, type_hint)| {
                    format!("{} {}", type_hint, name)
                }).collect::<Vec<_>>().join(", ");
                self.emit(format!("{} {}({}) {{", return_type_hint, name, args));
                match &**body {
                    IR::Value { value } => {
                        self.emit("return ");
                        self.gen_value(&value);
                        self.emit(";");
                    },
                    IR::Do { body } => {
                        for (i, node) in body.iter().enumerate() {
                            if i == body.len() - 1 {
                                self.emit("return ");
                            };
                            self.gen_ir(node);
                            self.emit(";");
                        }
                    },
                    IR::Binary { op, left, right } => {
                        self.emit("return ");
                        self.gen_ir(left);
                        self.emit(op);
                        self.gen_ir(right);
                        self.emit(";");
                    },
                    _ => todo!(),
                }
                self.emit("}\n");
            },
            IR::Call { name, args } => {
                match name.as_str() {
                    "puts" => {
                        self.emit("printf(");
                        self.gen_ir(&args[0]);
                        self.emit(")");
                    },
                    _ => {
                        self.emit(format!("{}(", name));
                        for (i, arg) in args.iter().enumerate() {
                            if i != 0 {
                                self.emit(", ");
                            }
                            self.gen_ir(arg);
                        }
                        self.emit(")");
                    }   
                }
            },
            IR::Value { value } => {
                self.gen_value(value);
            },
            IR::Binary { op, left, right } => {
                self.gen_ir(left);
                self.emit(op);
                self.gen_ir(right);
                self.emit(";");
            },
            _ => todo!()
        }
    }

    fn gen_value(&mut self, value: &Value) {
        match value {
            Value::Int(i) => self.emit(format!("{}", i)),
            Value::Float(f) => self.emit(format!("{}", f)),
            Value::Double(d) => self.emit(format!("{}", d)),
            Value::Bool(b) => self.emit(format!("{}", b)),
            Value::String(s) => self.emit(format!("\"{}\"", s)),
            Value::Ident(s) => self.emit(format!("{}", s)),
        }
    }

}