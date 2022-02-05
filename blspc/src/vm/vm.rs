use std::{io::{self, Read}, fmt::Display, fs::File};

use crate::vm::instr::{Instr::{self, *}, Type, Register};

pub enum Error {
    NoMainFunction,
    StackOverflow,
    UnknownFunction(String),
    UnknownFunctionCall(String),
    InvalidAriphmeticOperation,
    FileError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoMainFunction => write!(f, "Main function not found"),
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::UnknownFunction(name) => write!(f, "Unknown function: {}", name),
            Error::UnknownFunctionCall(function) => write!(f, "Unknown function call: {}", function),
            Error::InvalidAriphmeticOperation => write!(f, "Invalid ariphmetic operation"),
            Error::FileError(msg) => write!(f, "Could not open file: {}", msg),
        }
    }
}

#[derive(Debug)]
pub struct VM {
    instr_pointer: isize,
    jumped_from: isize,
    registers: Vec<Type>,
    stack: Vec<Type>,
    function_pointer: Vec<(String, isize)>, // (name, index)
}

pub type VMReturn = Result<(), Error>;

impl VM {
    pub fn new() -> Self {
        VM {
            instr_pointer: 0,
            jumped_from: 0,
            registers: vec![Type::Null; 1024],
            stack: Vec::new(),
            function_pointer: Vec::new(),
        }
    }
    
    pub fn run(&mut self, instrs: Vec<Instr>, debug: bool) -> VMReturn {
        let result: VMReturn;

        let mut found = false;
        for (idx, instr) in instrs.iter().enumerate() {
            match instr {
                Label { name } => {
                    if name == "function_main" { self.instr_pointer = idx as isize; found = true; }
                    self.function_pointer.push((name.clone(), idx as isize));
                },
                _ => {}
            }
        }
        if !found { return Err(Error::NoMainFunction); }

        'tco: loop {
            // std::thread::sleep(std::time::Duration::from_millis(1000));

            self.instr_pointer += 1;
            if self.instr_pointer - 1 == instrs.len() as isize {
                result = Ok(());
                break 'tco;
            }

            let instr = &instrs[(self.instr_pointer - 1) as usize];
            if debug { print_debug(self, &instr); }
            match instr {
                Load { address } => {
                    self.load(address)?;
                    continue 'tco;
                },
                Store { address } => {
                    let value = &self.stack.pop().unwrap();
                    self.store(address, value)?;
                    continue 'tco;
                },

                Call { function } => {
                    self.call(function)?;
                    continue 'tco;
                },

                Push { value } => {
                    self.push(value.trim().clone())?;
                    continue 'tco;
                },
                Pop { address } => {
                    let value = self.stack.pop();
                    self.pop(&address, &value.unwrap())?;
                    continue 'tco;
                },
                Swap => {
                    let top = self.stack.pop().unwrap();
                    let bottom = self.stack.pop().unwrap();
                    self.stack.push(top);
                    self.stack.push(bottom);
                    continue 'tco;
                },

                Add => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs + rhs)?)?;
                    continue 'tco;
                },
                Sub => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs - rhs)?)?;
                    continue 'tco;
                },
                Mul => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs * rhs)?)?;
                    continue 'tco;
                },
                Div => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs / rhs)?)?;
                    continue 'tco;
                },

                Not => {
                    let value = self.stack.pop().unwrap();
                    self.push((!value)?)?;
                    continue 'tco;
                },

                JumpLabel { to } => {
                    let pointer = self.get_function_pointer(to.to_string())?;
                    self.jumped_from = self.instr_pointer;
                    self.instr_pointer = pointer;
                    continue 'tco;
                },
                Jump { to } => {
                    self.instr_pointer += *to as isize + 1;
                    continue 'tco;
                },
                JumpIfFalse { to } => {
                    let cond = self.stack.pop().unwrap();
                    if cond == Type::Boolean(false) {
                        if *to < 0 { self.instr_pointer += *to as isize - 2; }
                        else { self.instr_pointer += *to as isize + 1; }
                        continue 'tco;
                    }
                },

                Equal => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push(Type::Boolean(lhs == rhs))?;
                    continue 'tco;
                },

                Return => {
                    if self.jumped_from == 0 { return Ok(()); }
                    self.instr_pointer = self.jumped_from;
                    self.jumped_from = 0;
                    continue 'tco;
                },
                Label { .. } => {},
                _ => { dbg!(instr); unimplemented!()},
            }
        }

        result
    }
    
    fn push(&mut self, value: Type) -> Result<(), Error> {
        if self.stack.len() >= 1024 {
            return Err(Error::StackOverflow);
        }
        Ok(self.stack.push(value))
    }
    
    fn pop(&mut self, address: &Register, value: &Type) -> Result<(), Error> {
        // TODO: Remove .clone()
        Ok(self.registers[address.value()] = value.clone())
    }

    fn store(&mut self, address: &Register, value: &Type) -> Result<(), Error> {
        Ok(self.registers[address.value()] = value.clone())
    }

    fn load(&mut self, address: &Register) -> Result<(), Error> {
        Ok(self.stack.push(self.registers[address.value()].clone()))
    }

    fn get_function_pointer(&mut self, name: String) -> Result<isize, Error> {
        for (idx, (n, _)) in self.function_pointer.iter().enumerate() {
            if n == &name {
                return Ok(idx as isize);
            }
        }
        Err(Error::UnknownFunction(name))
    }

    fn call(&mut self, function: &String) -> Result<(), Error> {
        match function.as_str() {
            "print" => {
                let value = self.stack.pop().unwrap();
                println!("{}", value.fmt());
                return Ok(());
            },
            "read" => {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().parse::<Type>().unwrap();
                self.stack.push(input);
                Ok(())
            },
            "slurp" => {
                let file_name = self.stack.pop().unwrap().fmt();
                let mut result = String::new();
                match File::open(file_name).and_then(|mut f| f.read_to_string(&mut result)) {
                    Ok(_) => Ok(self.stack.push(Type::String(result))),
                    Err(e) => Err(Error::FileError(e.to_string())),
                }
            }
            _ => { dbg!(function); Err(Error::UnknownFunctionCall(function.to_string())) },
        }
    }
}

fn print_debug(vm: &VM, curr_instr: &Instr) {
    // get all register that are not null
    let regs = vm.registers.iter().enumerate().filter(|(_, v)| !v.is_null()).collect::<Vec<_>>();
    println!("regis: {:?}", regs);
    println!("stack: {:?}", vm.stack);
    println!("currn: {} {}", vm.instr_pointer, curr_instr);
}