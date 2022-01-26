use crate::{vm::instr::*, compiler::parser::Sexpr::{self, *}};
pub struct Compiler {
    pub instructions: Vec<Instr>,
    pub register_pointer: usize,
    pub label_pointer: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Vec::new(),
            register_pointer: 1,
            label_pointer: 1,
        }
    }
    
    fn next_register(&mut self) -> Register {
        let r = Register { value: self.register_pointer };
        self.register_pointer += 1;
        r
    }

    fn current_register(&self) -> Register {
        Register { value: self.register_pointer - 1 }
    }

    fn next_label(&mut self) -> usize {
        let l = self.label_pointer;
        self.label_pointer += 1;
        l
    }

    fn current_label(&self) -> usize {
        self.label_pointer - 1
    }
    
    pub fn compile(&mut self, ast: Sexpr, depth: usize) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match ast {
            Cons(car, cdr) => {
                match *car {
                    Symbol(ref s) => {
                        match s.as_str() {
                            "do" => {
                                for c in cdr {
                                    result.append(&mut self.compile(c, depth + 1)?);
                                }
                            }
                            "if" => {
                                // TODO: Remove .clone()
                                let mut cond = self.compile(cdr[0].clone(), depth + 1)?;
                                let cond_register = self.current_register();

                                result.append(&mut cond);

                                result.push(Instr::JumpIfFalse {
                                    cond: cond_register,
                                    to: 999, // To be replaced later
                                    label: self.next_label(),
                                });
                                
                                let mut then = self.compile(cdr[1].clone(), depth + 1)?;
                                let jump_label = self.next_label();

                                let mut else_ = self.compile(cdr[2].clone(), depth + 1)?;
                                let else_label = self.current_label() - else_.len() + 1;

                                let idx = result.len() - 1;
                                match result[idx] {
                                    Instr::JumpIfFalse { cond: c, to: _, label: l } => {
                                        result[idx] = Instr::JumpIfFalse { cond: c, to: else_label, label: l, };
                                    }
                                    _ => unreachable!(),
                                }

                                result.append(&mut then);
                                result.push(Instr::Jump {
                                    to: self.current_label() + 1,
                                    label: jump_label,
                                });
                                result.append(&mut else_);
                            }
                            _ => {
                                result.append(&mut self.compile_intrinsic(s, &cdr, depth + 1)?);
                            }
                        }
                    }
                    _ => return Err(format!("Expected symbol, got {:?}", car)),
                }
            }
            _ => { result.append(&mut self.compile_atom(&ast, depth + 1)?); },
        }
        
        if depth == 0 {
            result.push(Instr::Store {
                address: self.next_register(),
                value: Type::Int(0),
                label: self.next_label(), 
            });
            result.push(Instr::Return {
                value: self.current_register(),
                label: self.next_label(),
            });
        }
        Ok(result)
    }
    
    fn compile_atom(&mut self, atom: &Sexpr, depth: usize) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match atom {
            Int(i) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::Int(*i),
                    label: self.next_label(),
                });
            }
            Float(f) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::Float(*f),
                    label: self.next_label(),
                });
            }
            Boolean(b) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::Boolean(*b),
                    label: self.next_label(),
                });
            }
            Str(s) => {
                let r = self.next_register();
                result.push(Instr::Store {
                    address: r,
                    value: Type::String(s.to_string()),
                    label: self.next_label(),
                });
            }
            _ => {
                result.append(&mut self.compile(atom.clone(), depth + 1)?);
            }
        }
        
        Ok(result)
    }

    fn compile_intrinsic(&mut self, intrinsic: &String, args: &[Sexpr], depth: usize) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match intrinsic.as_str() {
            "print" => {
                let mut arg = self.compile_atom(&args[0], depth + 1)?;
                result.append(&mut arg);
                let arg_pointer = self.current_register();
                
                let call_register = self.next_register();
                result.push(Instr::Store {
                    address: call_register,
                    value: Type::Int(1),
                    label: self.next_label(),
                });
                
                result.push(Instr::Call {
                    address: call_register,
                    args: arg_pointer,
                    label: self.next_label(),
                });
            },
            "add" | "+" => {
                let mut lhs = self.compile_atom(&args[0], depth + 1)?;
                let lhs_pointer = self.current_register();
                result.append(&mut lhs);

                let mut rhs = self.compile_atom(&args[1], depth + 1)?;
                let rhs_pointer = self.current_register();
                result.append(&mut rhs);

                let result_register = self.next_register();
                result.push(Instr::IAdd {
                    lhs: lhs_pointer,
                    rhs: rhs_pointer,
                    to: result_register,
                    label: self.next_label(),
                });
            },
            "sub" | "-" => {
                let mut lhs = self.compile_atom(&args[0], depth + 1)?;
                let lhs_pointer = self.current_register();
                result.append(&mut lhs);

                let mut rhs = self.compile_atom(&args[1], depth + 1)?;
                let rhs_pointer = self.current_register();
                result.append(&mut rhs);

                let result_register = self.next_register();
                result.push(Instr::ISub {
                    lhs: lhs_pointer,
                    rhs: rhs_pointer,
                    to: result_register,
                    label: self.next_label(),
                });
            },
            "mul" | "*" => {
                let mut lhs = self.compile_atom(&args[0], depth + 1)?;
                let lhs_pointer = self.current_register();
                result.append(&mut lhs);

                let mut rhs = self.compile_atom(&args[1], depth + 1)?;
                let rhs_pointer = self.current_register();
                result.append(&mut rhs);

                let result_register = self.next_register();
                result.push(Instr::IMul {
                    lhs: lhs_pointer,
                    rhs: rhs_pointer,
                    to: result_register,
                    label: self.next_label(),
                });
            },
            "div" | "/" => {
                let mut lhs = self.compile_atom(&args[0], depth + 1)?;
                let lhs_pointer = self.current_register();
                result.append(&mut lhs);

                let mut rhs = self.compile_atom(&args[1], depth + 1)?;
                let rhs_pointer = self.current_register();
                result.append(&mut rhs);

                let result_register = self.next_register();
                result.push(Instr::IDiv {
                    lhs: lhs_pointer,
                    rhs: rhs_pointer,
                    to: result_register,
                    label: self.next_label(),
                });
            },
            _ => return Err(format!("Unknown intrinsic: {}", intrinsic)),
        }
        
        Ok(result)
    }
}