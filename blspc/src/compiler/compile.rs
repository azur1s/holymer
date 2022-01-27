use crate::{vm::instr::*, compiler::parser::Sexpr::{self, *}};
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
    
    fn current_register(&self) -> Register {
        Register { value: self.register_pointer - 1 }
    }
    
    pub fn compile(&mut self, src: Sexpr) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        let comp = src.clone(); // Used for commenting
        
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
                                    result.push(Instr::Comment { text: format!("--- {}", comp) });
                                    let function_name = match &cdr[0] {
                                        Symbol(ref name) => format!("function_{}", name.clone()),
                                        _ => return Err(format!("Expected function name, got {}", cdr[0])),
                                    };
                                    let body = &cdr[1];
                                    
                                    result.push(Instr::Label{ name: function_name });
                                    result.append(&mut self.compile(body.clone())?);
                                    result.push(Instr::Return);
                                },
                                "print" => {
                                    result.append(&mut self.compile(cdr[0].clone())?);
                                    let to = self.next_register();
                                    let call_register = self.next_register();
                                    
                                    result.push(Instr::Pop { address: to });
                                    result.push(Instr::Store {
                                        address: call_register,
                                        value: Type::Int(1),
                                    });
                                    result.push(Instr::Call {
                                        address: call_register,
                                        args: to,
                                    });
                                }
                                _ => { dbg!(call); unimplemented!() },
                            } // End `match call`
                        }, // End `Symbol(call)`
                        _ => { dbg!(car); unimplemented!() },
                    } // End `match car`
                }, // End `Cons(car, cdr)`
                _ => { result.append(&mut self.compile_atom(src)?); },
            } // End `match src`
            
            break 'tco;
        } // End `loop`
        
        Ok(result)
    }
    
    fn compile_atom(&mut self, atom: Sexpr) -> Result<Vec<Instr>, String> {
        let mut result = Vec::new();
        let comp = atom.clone(); // Used for commenting
        
        match atom {
            Int(i) => {
                result.push(Instr::Comment { text: format!("----- {}", comp) });
                result.push(Instr::Push { value: Type::Int(i) });
            },
            Float(f) => {
                result.push(Instr::Comment { text: format!("----- {}", comp) });
                result.push(Instr::Push { value: Type::Float(f) });
            },
            Str(s) => {
                result.push(Instr::Comment { text: format!("----- {}", comp) });
                result.push(Instr::Push { value: Type::String(s) });
            },
            Boolean(b) => {
                result.push(Instr::Comment { text: format!("----- {}", comp) });
                result.push(Instr::Push { value: Type::Boolean(b) });
            },
            Symbol(s) => {
                result.push(Instr::Comment { text: format!("----- {} variable", comp) });
                result.push(Instr::Jump { to: format!("function_{}", s), });
            },
            _ => { dbg!(atom); unimplemented!() },
        }
        
        Ok(result)
    }
}