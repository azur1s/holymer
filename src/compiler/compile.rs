use crate::{
    compiler::lib::{Type, Register, Instr},
    parser::Sexpr::{self, *},
};

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
                            _ => {
                                result.append(&mut self.compile_intrinsic(s, &cdr)?);
                            }
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

    fn compile_intrinsic(&mut self, intrinsic: &String, args: &[Sexpr]) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        
        match intrinsic.as_str() {
            "print" => {
                let mut arg = self.compile_atom(&args[0])?;
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
            }
            _ => return Err(format!("Unknown intrinsic: {}", intrinsic)),
        }
        
        Ok(result)
    }
}