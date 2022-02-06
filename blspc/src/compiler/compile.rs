use crate::{vm::{instr::*, types::Type}, compiler::parser::Sexpr::{self, *}};

pub struct Compiler {
    // Compiled instructions
    pub instructions: Vec<Instr>,
    // Compiled variables's register address
    pub variables: Vec<(String, Register)>,
    // Current register index
    pub register_pointer: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Vec::new(),
            variables: Vec::new(),
            register_pointer: 1,
        }
    }
    
    fn next_register(&mut self) -> Register {
        let r = Register { value: self.register_pointer };
        self.register_pointer += 1;
        r
    }
    
    pub fn compile(&mut self, src: Sexpr) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        'tco: loop {
            match src {
                Cons(car, cdr) => {
                    match *car {
                        Symbol(ref call) => {
                            match call.as_str() {
                                "do" => {
                                    for c in cdr {
                                        result.append(&mut self.compile(c)?);
                                    }
                                },
                                "fun" => {
                                    let function_name = match &cdr[0] {
                                        Symbol(ref name) => format!("function_{}", name.clone()),
                                        _ => return Err(format!("Expected function name, got {}", cdr[0])),
                                    };
                                    let body = &cdr[1];
                                    
                                    result.push(Instr::Label { name: function_name });
                                    result.append(&mut self.compile(body.clone())?);
                                    result.push(Instr::Return);
                                },
                                "if" => {
                                    let mut cond = self.compile(cdr[0].clone())?;
                                    result.append(&mut cond);

                                    let mut then = self.compile(cdr[1].clone())?;
                                    let mut else_ = self.compile(cdr[2].clone())?;

                                    result.push(Instr::JumpIfFalse { to: len(&then) as isize + 1 }); // +1 for the jump instr
                                    result.append(&mut then);
                                    result.push(Instr::Jump { to: len(&else_) as isize });
                                    result.append(&mut else_);
                                },
                                "def" => {
                                    let var_name = match &cdr[0] {
                                        Symbol(ref name) => name.clone(),
                                        _ => return Err(format!("Expected variable name, got {}", cdr[0])),
                                    };

                                    if let Some(v) = self.variables.iter().find(|v| v.0 == var_name) {
                                        let r = v.1;
                                        self.variables.retain(|v| v.0 != var_name);
                                        self.variables.push((var_name, r));
                                        result.append(&mut self.compile(cdr[1].clone())?);
                                        result.push(Instr::Store { address: r });
                                    } else {
                                        let var_pointer = self.next_register();
                                        self.variables.push((var_name, var_pointer));
                                        result.append(&mut self.compile(cdr[1].clone())?);
                                        result.push(Instr::Store { address: var_pointer });
                                    }
                                },
                                "while" => {
                                    let mut cond = self.compile(cdr[0].clone())?;
                                    let mut body = self.compile(cdr[1].clone())?;

                                    let jump_length = (len(&body) as isize) + (len(&cond) as isize);

                                    result.append(&mut cond.clone());
                                    result.push(Instr::JumpIfFalse { to: jump_length + 1 });
                                    result.append(&mut body);
                                    result.append(&mut cond);
                                    result.push(Instr::Not);
                                    result.push(Instr::JumpIfFalse { to: -jump_length });
                                },
                                _ => {
                                    result.append(&mut self.compile_intrinsic(call, &cdr)?);
                                }
                            } // End `match call`
                        }, // End `Symbol(call)`
                        _ => { dbg!(car); unimplemented!() },
                    } // End `match car`
                }, // End `Cons(car, cdr)`
                _ => { result.append(&mut self.compile_atom(&src)?); },
            } // End `match src`
            
            break 'tco;
        } // End `loop`
        
        Ok(result)
    }

    fn compile_intrinsic(&mut self, intrinsic: &String, args: &[Sexpr]) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match intrinsic.as_str() {
            "print" => {
                result.append(&mut self.compile(args[0].clone())?);
                result.push(Instr::Call { function: "print".to_string() });
            },
            "read" => { result.push(Instr::Call { function: "read".to_string() }); },
            "slurp" => {
                result.append(&mut self.compile(args[0].clone())?);
                result.push(Instr::Call { function: "slurp".to_string() });
            },

            "add" | "+" => {
                let mut lhs = self.compile_atom(&args[0])?;
                result.append(&mut lhs);
                
                let mut rhs = self.compile_atom(&args[1])?;
                result.append(&mut rhs);
                
                result.push(Instr::Add);
            },
            "sub" | "-" => {
                let mut lhs = self.compile_atom(&args[0])?;
                result.append(&mut lhs);
                
                let mut rhs = self.compile_atom(&args[1])?;
                result.append(&mut rhs);
                
                result.push(Instr::Swap);
                result.push(Instr::Sub);
            },
            "mul" | "*" => {
                let mut lhs = self.compile_atom(&args[0])?;
                result.append(&mut lhs);
                
                let mut rhs = self.compile_atom(&args[1])?;
                result.append(&mut rhs);
                
                result.push(Instr::Mul);
            },
            "div" | "/" => {
                let mut lhs = self.compile_atom(&args[0])?;
                result.append(&mut lhs);
                
                let mut rhs = self.compile_atom(&args[1])?;
                result.append(&mut rhs);
                
                result.push(Instr::Swap);
                result.push(Instr::Div);
            },
            "equal" | "=" => {
                let mut lhs = self.compile_atom(&args[0])?;
                result.append(&mut lhs);
                
                let mut rhs = self.compile_atom(&args[1])?;
                result.append(&mut rhs);
                
                result.push(Instr::Equal);
            },
            "not" | "!" => {
                let mut lhs = self.compile_atom(&args[0])?;
                result.append(&mut lhs);
                
                result.push(Instr::Not);
            },
            _ => {
                result.push(Instr::Comment { text: format!("`{}` function", intrinsic) });
                result.push(Instr::JumpLabel { to: format!("function_{}", intrinsic), });
            }
        }

        Ok(result)
    }
    
    fn compile_atom(&mut self, atom: &Sexpr) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match atom {
            Int(i) => {
                result.push(Instr::Push { value: Type::Int(*i) });
            },
            Float(f) => {
                result.push(Instr::Push { value: Type::Float(*f) });
            },
            Str(s) => {
                result.push(Instr::Push { value: Type::String(s.to_string()) });
            },
            Boolean(b) => {
                result.push(Instr::Push { value: Type::Boolean(*b) });
            },
            Symbol(s) => {
                let var_pointer = match self.variables.iter().find(|&(ref name, _)| name == s) {
                    Some((_, pointer)) => *pointer,
                    None => return Err(format!("Undefined variable {}", s)),
                };
                result.push(Instr::Comment { text: format!("`{}` variable", s) });
                result.push(Instr::Load { address: var_pointer });
            },
            _ => { result.append(&mut self.compile(atom.clone())?); }
        }
        
        Ok(result)
    }
}

fn len(vec: &Vec<Instr>) -> usize {
    let mut result = 0;
    for i in vec {
        match i {
            Instr::Comment { .. } => {},
            Instr::Label { .. } => {},
            _ => { result += 1; },
        }
    }
    result
}