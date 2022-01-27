use std::{fmt::Display, str::FromStr};

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
            Type::Int(i)     => write!(f, ":{}", i),
            Type::Float(fl)  => write!(f, ":{}", fl),
            Type::Boolean(b) => write!(f, ":{}", b),
            Type::String(s)  => write!(f, "$\"{}\"", s),
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("$") && !s.starts_with(":") {
            return Err(format!("Invalid literal: {}", s));
        }

        let s = &s[1..];
        match s {
            "true"  => Ok(Type::Boolean(true)),
            "false" => Ok(Type::Boolean(false)),
            _ => {
                let fl = s.parse::<f64>();
                if fl.is_ok() {
                    Ok(Type::Float(fl.unwrap()))
                } else {
                    let i = s.parse::<i64>();
                    if i.is_ok() {
                        Ok(Type::Int(i.unwrap()))
                    } else {
                        Ok(Type::String(s.to_string()))
                    }
                }
            }
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

impl FromStr for Register {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s[1..].parse::<usize>().map_err(|_| ())?;
        Ok(Register { value })
    }
}

/// Instructions for the assembler.
#[derive(Clone, Debug)]
pub enum Instr {
    // Load a literal value onto the stack.
    // Load { address: Register, label: usize },
    // Store a literal value into a register.
    Store { address: Register, value: Type, label: usize },
    // Call intrinsic function.
    Call { address: Register, args: Register, label: usize },
    // Stack operations.
    Push { value: Type, label: usize }, Pop { address: Register, label: usize },
    // Stack arithmetic.
    Add { label: usize }, Sub { label: usize },
    Mul { label: usize }, Div { label: usize },
    // Jumping
    Jump { to: usize, label: usize },
    PopJumpIfFalse { to: usize, label: usize },

    Return { value: Register, label: usize },
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Instr::Load { address, label }         => write!(f, "{}: LOAD {}", label, address),
            Instr::Store { address, value , label} => write!(f, "{} STORE {} {}", label, address, value),
            Instr::Call { address, args, label }   => write!(f, "{} CALL {} {}", label, address, args),
            Instr::Push { value, label }           => write!(f, "{} PUSH {}", label, value),
            Instr::Pop { address, label }          => write!(f, "{} POP {}", label, address),
            Instr::Add { label }                   => write!(f, "{} ADD", label),
            Instr::Sub { label }                   => write!(f, "{} SUB", label),
            Instr::Mul { label }                   => write!(f, "{} MUL", label),
            Instr::Div { label }                   => write!(f, "{} DIV", label),
            Instr::Jump { to, label }              => write!(f, "{} JMP {}", label, to),
            Instr::PopJumpIfFalse { to, label }    => write!(f, "{} POP_JUMP_IF_FALSE {}", label, to),
            Instr::Return { value, label }         => write!(f, "{} RETURN {}", label, value),
        }
    }
}