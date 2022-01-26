use std::fmt::Display;

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
    // Load a literal value onto the stack.
    Load { address: Register },
    // Store a literal value into a register.
    Store { address: Register, value: Type, },
    // Call intrinsic function.
    Call { address: Register, args: Register },
    // Immediate arithmetic.
    IAdd { lhs: Register, rhs: Register, to: Register, },
    ISub { lhs: Register, rhs: Register, to: Register, },
    IMul { lhs: Register, rhs: Register, to: Register, },
    IDiv { lhs: Register, rhs: Register, to: Register, },
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Instr::Load { address } => write!(f, "LOAD {}", address),
            Instr::Store { address, value } => write!(f, "STORE {} {}", address, value),
            Instr::Call { address, args } => write!(f, "CALL {} {}", address, args),
            Instr::IAdd { lhs, rhs, to } => write!(f, "IADD {} {} {}", lhs, rhs, to),
            Instr::ISub { lhs, rhs, to } => write!(f, "ISUB {} {} {}", lhs, rhs, to),
            Instr::IMul { lhs, rhs, to } => write!(f, "IMUL {} {} {}", lhs, rhs, to),
            Instr::IDiv { lhs, rhs, to } => write!(f, "IDIV {} {} {}", lhs, rhs, to),
        }
    }
}