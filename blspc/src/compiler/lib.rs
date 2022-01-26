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
    Load { address: Register, label: usize },
    // Store a literal value into a register.
    Store { address: Register, value: Type, label: usize },
    // Call intrinsic function.
    Call { address: Register, args: Register, label: usize },
    // Immediate arithmetic.
    IAdd { lhs: Register, rhs: Register, to: Register, label: usize },
    ISub { lhs: Register, rhs: Register, to: Register, label: usize },
    IMul { lhs: Register, rhs: Register, to: Register, label: usize },
    IDiv { lhs: Register, rhs: Register, to: Register, label: usize },
    // Jumping
    Jump { to: usize, label: usize },
    JumpIfFalse { cond: Register, to: usize, label: usize },

    Return { value: Register, label: usize },
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Instr::Load { address, label }         => write!(f, "{}: LOAD {}", label, address),
            Instr::Store { address, value , label} => write!(f, "{}: STORE {} {}", label, address, value),
            Instr::Call { address, args, label }   => write!(f, "{}: CALL {} {}", label, address, args),
            Instr::IAdd { lhs, rhs, to, label }    => write!(f, "{}: IADD {} {} {}", label, lhs, rhs, to),
            Instr::ISub { lhs, rhs, to, label }    => write!(f, "{}: ISUB {} {} {}", label, lhs, rhs, to),
            Instr::IMul { lhs, rhs, to, label }    => write!(f, "{}: IMUL {} {} {}", label, lhs, rhs, to),
            Instr::IDiv { lhs, rhs, to, label }    => write!(f, "{}: IDIV {} {} {}", label, lhs, rhs, to),
            Instr::Jump { to, label }              => write!(f, "{}: JUMP {}", label, to),
            Instr::JumpIfFalse { cond, to, label } => write!(f, "{}: JUMP_IF_FALSE {} {}", label, cond, to),
            Instr::Return { value, label }         => write!(f, "{}: RETURN {}", label, value),
        }
    }
}