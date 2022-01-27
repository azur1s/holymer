use std::fmt::Display;

use crate::vm::instr::{Instr::{self, *}, Type, Register};

pub enum Error {
    StackOverflow,
    UnknownFunctionCall(isize, isize),
    InvalidAriphmeticOperation,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::UnknownFunctionCall(l, e) => write!(f, "Unknown function call at {}: {}", l, e),
            Error::InvalidAriphmeticOperation => write!(f, "Invalid ariphmetic operation"),
        }
    }
}

#[derive(Debug)]
pub struct VM {
    pub instr_pointer: isize,
    pub registers: Vec<Type>,
    pub stack: Vec<Type>,
}

pub type VMReturn = Result<(), Error>;

impl VM {
    pub fn new() -> Self {
        VM {
            instr_pointer: 0,
            registers: vec![Type::Null; 1024],
            stack: Vec::new(),
        }
    }
    
    pub fn run(&mut self, instrs: Vec<Instr>, debug: bool) -> VMReturn {
        'tco: loop {
            self.instr_pointer += 1;
            if self.instr_pointer - 1 == instrs.len() as isize { return Ok(()); }
            
            let instr = &instrs[self.instr_pointer as usize - 1];
            if debug { print_debug(&self, instr); }
            match instr {
                Store { address, value, .. } => {
                    self.store(&address, &value)?;
                    continue 'tco;
                },
                Call { address, args, .. } => {
                    let args = &self.registers[args.value()];
                    let address = &self.registers[address.value()];
                    call(address, args, self.instr_pointer)?;
                    continue 'tco;
                },
                Push { value, .. } => {
                    self.push(value.trim().clone())?;
                    continue 'tco;
                },
                Pop { address, .. } => {
                    let value = self.stack.pop();
                    self.store(&address, &value.unwrap())?;
                    continue 'tco;
                },
                Add { .. } => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs + rhs)?)?;
                    continue 'tco;
                },
                Sub { .. } => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs - rhs)?)?;
                    continue 'tco;
                },
                Mul { .. } => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs * rhs)?)?;
                    continue 'tco;
                },
                Div { .. } => {
                    let lhs = self.stack.pop().unwrap();
                    let rhs = self.stack.pop().unwrap();
                    self.push((lhs / rhs)?)?;
                    continue 'tco;
                },
                Jump { to, .. } => {
                    self.instr_pointer = *to as isize - 1;
                    continue 'tco;
                },
                PopJumpIfFalse { to, .. } => {
                    let value = self.stack.pop().unwrap();
                    if !value.as_bool() { self.instr_pointer = *to as isize - 1; }
                    continue 'tco;
                },
                Return { .. } => return Ok(()),
            };
        }
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