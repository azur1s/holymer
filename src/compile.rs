use std::fmt::Display;

use crate::parser::Sexpr::{self, *};

/// Literal types for the assembler.
#[derive(Clone, Debug)]
pub enum Type {
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Int(i)     => write!(f, "${}", i),
            Type::Float(fl)  => write!(f, "${}", fl),
            Type::Boolean(b) => write!(f, "${}", b),
            Type::String(s)  => write!(f, "$\"{}\"", s),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Register { pub value: usize }

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "r{}", self.value)
    }
}

/// Instructions for the assembler.
#[derive(Clone, Debug)]
pub enum Instr {
    // Load a literal value onto the stack;
    Load { address: Register },
    // Store a literal value into a register.
    Store { address: Register, value: Type, },
    // Call intrinsic function.
    Call { address: Register, args: Register },
    // Stack arithmetic.
    Add, Sub, Mul, Div,
    // Immediate arithmetic.
    IAdd { lhs: Register, rhs: Register, to: Register, },
    ISub { lhs: Register, rhs: Register, to: Register, },
    IMul { lhs: Register, rhs: Register, to: Register, },
    IDiv { lhs: Register, rhs: Register, to: Register, },
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Instr::Load { address } => write!(f, "load {}", address),
            Instr::Store { address, value } => write!(f, "store {} {}", address, value),
            Instr::Call { address, args } => write!(f, "call {} {}", address, args),
            Instr::Add => write!(f, "add"),
            Instr::Sub => write!(f, "sub"),
            Instr::Mul => write!(f, "mul"),
            Instr::Div => write!(f, "div"),
            Instr::IAdd { lhs, rhs, to } => write!(f, "iadd {} {} {}", lhs, rhs, to),
            Instr::ISub { lhs, rhs, to } => write!(f, "isub {} {} {}", lhs, rhs, to),
            Instr::IMul { lhs, rhs, to } => write!(f, "imul {} {} {}", lhs, rhs, to),
            Instr::IDiv { lhs, rhs, to } => write!(f, "idiv {} {} {}", lhs, rhs, to),
        }
    }
}

pub struct Compiler {
    pub instructions: Vec<Instr>,
    pub register_pointer: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Vec::new(),
            register_pointer: 1,
        }
    }
    
    fn next_register(&mut self) -> Register {
        let r = Register { value: self.register_pointer };
        self.register_pointer += 1;
        r
    }

    fn current_pointer(&self) -> Register {
        Register { value: self.register_pointer - 1 }
    }
    
    pub fn compile(&mut self, ast: Sexpr) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match ast {
            Cons(car, cdr) => {
                match *car {
                    Symbol(ref s) => {
                        match s.as_str() {
                            "do" => {
                                for c in cdr {
                                    result.append(&mut self.compile(c)?);
                                }
                            }
                            "print" => {
                                let mut arg = self.compile_atom(&cdr[0])?;
                                result.append(&mut arg);
                                let arg_pointer = self.current_pointer();
                                
                                let call_register = self.next_register();
                                result.push(Instr::Store {
                                    address: call_register,
                                    value: Type::Int(1),
                                });
                                
                                result.push(Instr::Call {
                                    address: call_register,
                                    args: arg_pointer,
                                });
                            },
                            _ => return Err(format!("Unknown symbol: {}", s)),
                        }
                    }
                    _ => return Err(format!("Expected symbol, got {:?}", car)),
                }
            }
            _ => { dbg!(ast); unimplemented!() }
        }
        
        Ok(result)
    }
    
    fn compile_atom(&mut self, atom: &Sexpr) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match atom {
            Int(i) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::Int(*i),
                });
            }
            Float(f) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::Float(*f),
                });
            }
            Boolean(b) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::Boolean(*b),
                });
            }
            Str(s) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::String(s.to_string()),
                });
            }
            _ => return Err(format!("Expected atom, got {:?}", atom)),
        }
        
        Ok(result)
    }
}