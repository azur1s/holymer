use std::fmt::Display;

use crate::vm::instr::{Instr::{self, *}, Type, Register};

pub enum Error {
    NoMainFunction,
    StackOverflow,
    UnknownFunction(String),
    UnknownFunctionCall(isize, isize),
    InvalidAriphmeticOperation,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoMainFunction => write!(f, "Main function not found"),
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::UnknownFunction(name) => write!(f, "Unknown function: {}", name),
            Error::UnknownFunctionCall(l, e) => write!(f, "Unknown function call at {}: {}", l, e),
            Error::InvalidAriphmeticOperation => write!(f, "Invalid ariphmetic operation"),
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
            self.instr_pointer += 1;
            if self.instr_pointer - 1 == instrs.len() as isize {
                result = Ok(());
                break 'tco;
            }

            let instr = &instrs[(self.instr_pointer - 1) as usize];
            if debug { print_debug(self, &instr); }
            match instr {
                Store { address, value, .. } => {
                    self.store(&address, &value)?;
                    continue 'tco;
                },
                Call => {
                    let index = &self.stack.pop().unwrap();
                    let args = &self.stack.pop().unwrap();
                    call(index, args, self.instr_pointer)?;
                    continue 'tco;
                },
                Push { value } => {
                    self.push(value.trim().clone())?;
                    continue 'tco;
                },
                Pop { address } => {
                    let value = self.stack.pop();
                    self.store(&address, &value.unwrap())?;
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
                Jump { to } => {
                    let pointer = self.get_function_pointer(to.to_string())?;
                    self.jumped_from = self.instr_pointer;
                    self.instr_pointer = pointer;
                    continue 'tco;
                },
                Return => {
                    if self.jumped_from == 0 { return Ok(()); }
                    self.instr_pointer = self.jumped_from;
                    self.jumped_from = 0;
                    continue 'tco;
                },
                Label { .. } => {},
                _ => unimplemented!(),
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
    
    fn store(&mut self, address: &Register, value: &Type) -> Result<(), Error> {
        // TODO: Remove .clone()
        Ok(self.registers[address.value()] = value.clone())
    }

    fn get_function_pointer(&mut self, name: String) -> Result<isize, Error> {
        for (idx, (n, _)) in self.function_pointer.iter().enumerate() {
            if n == &name {
                return Ok(idx as isize);
            }
        }
        Err(Error::UnknownFunction(name))
    }
}

fn print_debug(vm: &VM, curr_instr: &Instr) {
    // get all register that are not null
    let regs = vm.registers.iter().enumerate().filter(|(_, v)| !v.is_null()).collect::<Vec<_>>();
    println!("regis: {:?}", regs);
    println!("stack: {:?}", vm.stack);
    println!("currn: {} {}", vm.instr_pointer, curr_instr);
}

fn call(index: &Type, args: &Type, line: isize) -> Result<(), Error> {
    match index {
        Type::Int(i) => {
            match i {
                0 => Err(Error::UnknownFunctionCall(line, 0)),
                1 => {
                    println!("{}", args.fmt());
                    Ok(())
                },
                _ => Err(Error::UnknownFunctionCall(line, *i as isize)),
            }
        }
        _ => {dbg!(index); Err(Error::UnknownFunctionCall(line, -1))},
    }
}