use std::fmt::Display;

use crate::parser::Sexpr::{self, *};

#[derive(Clone, Debug, Copy)]
pub struct Register { value: i64 }

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Literal types for the bytecode.
#[derive(Clone, Debug)]
pub enum Type {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Int(i) => write!(f, "{}", i),
            Type::Float(fl) => write!(f, "{}", fl),
            Type::Bool(b) => write!(f, "{}", b),
            Type::String(s) => write!(f, "{}", s),
        }
    }
}

/// Instructions for the bytecode.
#[derive(Clone, Debug)]
pub enum Instr {
    /// Call(Function Index, Arguments)
    Call(usize, [Register; 6]),

    /// Stack manipulators
    Push(usize, Type), Pop(usize, Register),

    JumpIfFalse(usize, Register, usize),
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Instr::Call(idx, args)         => write!(f, "{} call {} {} {} {} {} {}", idx, args[0], args[1], args[2], args[3], args[4], args[5]),
            Instr::Push(idx, t)            => write!(f, "{} push {}", idx, t),
            Instr::Pop(idx, r)             => write!(f, "{} pop {}", idx, r),
            Instr::JumpIfFalse(idx, r, to) => write!(f, "{} jmpf {} {}", idx, r, to),
        }
    }
}

pub struct Compiler {
    /// The bytecode.
    pub bytecode: Vec<Instr>,
    /// The stack.
    pub stack: Vec<Type>,
    /// The current register index.
    pub register: Register,
    /// The current label index.
    pub label: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            bytecode: Vec::new(),
            stack: Vec::new(),
            register: Register { value: 1 },
            label: 0,
        }
    }

    fn next_register(&mut self) -> Register {
        let r = self.register;
        self.register.value += 1;
        r
    }

    fn next_label(&mut self) -> usize {
        let l = self.label;
        self.label += 1;
        l
    }

    pub fn compile_sexpr(&mut self, ast: Sexpr) -> Vec<Instr> {
        let mut result: Vec<Instr> = Vec::new();
    
        match ast {
            Cons(car, cdr) => {
                match *car {
                    Symbol(f) => {
                        match f.as_str() {
                            "do" => {
                                for c in cdr {
                                    result.append(&mut self.compile_sexpr(c));
                                }
                            },
                            "print" => {
                                let function_register = self.next_register();
                                result.push(Instr::Push(self.next_label(), Type::Int(1)));
                                result.push(Instr::Pop(self.next_label(), function_register));

                                let arg = &cdr[0];
                                let instrs = &mut self.compile_ast(arg);
                                result.append(&mut instrs.clone());

                                let arg_register = match instrs.last().unwrap() {
                                    Instr::Pop(_, r) => *r,
                                    _ => panic!("Expected mov instruction in `print`"),
                                };

                                result.push(
                                    Instr::Call(self.next_label(), [
                                        function_register,
                                        arg_register,
                                        Register { value: 0 },
                                        Register { value: 0 },
                                        Register { value: 0 },
                                        Register { value: 0 }
                                        ])
                                );
                            },
                            _ => todo!(),
                        }
                    },
                    _ => todo!(),
                }
            },
            _ => todo!()
        }

        result
    }

    fn compile_ast(&mut self, ast: &Sexpr) -> Vec<Instr> {
        let mut result = Vec::new();
        match ast {
            Str(s) => {
                result.push(Instr::Push(self.next_label(), Type::String(format!("\"{}\"", s))));
                result.push(Instr::Pop(self.next_label(), self.next_register()));
            },
            _ => todo!()
        }
        result
    }
}