use std::{fmt::Display, str::FromStr, ops::{Add, Sub, Mul, Div}};

use crate::vm::vm::Error::{self, InvalidAriphmeticOperation};

/// Literal types for the assembler.
#[derive(Clone, Debug)]
pub enum Type {
    Null,
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Type>),
}

impl Type {
    pub fn as_bool(&self) -> bool {
        match self {
            Type::Null => false,
            Type::Boolean(b) => *b,
            Type::Int(i) => *i != 0,
            Type::Float(f) => *f != 0.0,
            Type::String(s) => !s.is_empty(),
            Type::Array(a) => !a.is_empty(),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Type::Null => true,
            _ => false,
        }
    }

    pub fn trim(&self) -> Type {
        match self {
            Type::Null => Type::Null,
            Type::Int(i) => Type::Int(*i),
            Type::Float(f) => Type::Float(*f),
            Type::Boolean(b) => Type::Boolean(*b),
            Type::String(s) => Type::String(s[1..s.len() - 1].to_string()),
            Type::Array(a) => Type::Array(a.iter().map(|t| t.trim()).collect()),
        }
    }

    pub fn fmt(&self) -> String {
        match self {
            Type::Null => "null".to_string(),
            Type::Int(i) => i.to_string(),
            Type::Float(f) => f.to_string(),
            Type::Boolean(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            Type::String(s) => s.clone(),
            Type::Array(a) => {
                let mut s = "[".to_string();
                for (i, t) in a.iter().enumerate() {
                    s.push_str(&t.fmt());
                    if i < a.len() - 1 { s.push_str(", "); }
                }
                s.push_str("]");
                s
            }
        }
    }
}

impl Add for Type {
    type Output = Result<Type, Error>;

    fn add(self, other: Type) -> Result<Type, Error> {
        match (self, other) {
            (Type::Int(lhs), Type::Int(rhs)) => Ok(Type::Int(lhs + rhs)),
            (Type::Int(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs as f64 + rhs)),
            (Type::Float(lhs), Type::Int(rhs)) => Ok(Type::Float(lhs + rhs as f64)),
            (Type::Float(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs + rhs)),
            (Type::String(lhs), Type::String(rhs)) => Ok(Type::String(format!("{}{}", lhs, rhs))),
            _ => Err(InvalidAriphmeticOperation),
        }
    }
}

impl Sub for Type {
    type Output = Result<Type, Error>;

    fn sub(self, other: Type) -> Result<Type, Error> {
        match (self, other) {
            (Type::Int(lhs), Type::Int(rhs)) => Ok(Type::Int(lhs - rhs)),
            (Type::Int(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs as f64 - rhs)),
            (Type::Float(lhs), Type::Int(rhs)) => Ok(Type::Float(lhs - rhs as f64)),
            (Type::Float(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs - rhs)),
            _ => Err(InvalidAriphmeticOperation),
        }
    }
}

impl Mul for Type {
    type Output = Result<Type, Error>;

    fn mul(self, other: Type) -> Result<Type, Error> {
        match (self, other) {
            (Type::Int(lhs), Type::Int(rhs)) => Ok(Type::Int(lhs * rhs)),
            (Type::Int(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs as f64 * rhs)),
            (Type::Float(lhs), Type::Int(rhs)) => Ok(Type::Float(lhs * rhs as f64)),
            (Type::Float(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs * rhs)),
            _ => Err(InvalidAriphmeticOperation),
        }
    }
}

impl Div for Type {
    type Output = Result<Type, Error>;

    fn div(self, other: Type) -> Result<Type, Error> {
        match (self, other) {
            (Type::Int(lhs), Type::Int(rhs)) => Ok(Type::Int(lhs / rhs)),
            (Type::Int(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs as f64 / rhs)),
            (Type::Float(lhs), Type::Int(rhs)) => Ok(Type::Float(lhs / rhs as f64)),
            (Type::Float(lhs), Type::Float(rhs)) => Ok(Type::Float(lhs / rhs)),
            _ => Err(InvalidAriphmeticOperation),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Null       => write!(f, ":NULL"),
            Type::Int(i)     => write!(f, ":{}", i),
            Type::Float(fl)  => write!(f, ":{}", fl),
            Type::Boolean(b) => write!(f, ":{}", b),
            Type::String(s)  => write!(f, "$\"{}\"", s),
            Type::Array(a)   => {
                write!(f, "[[ ")?;
                for (i, t) in a.iter().enumerate() {
                    write!(f, "{}", t)?;
                    if i < a.len() - 1 { write!(f, ", ")?; }
                }
                write!(f, " ]]")
            }
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
                let i = s.parse::<i64>();
                if i.is_ok() {
                    Ok(Type::Int(i.unwrap()))
                } else {
                    let fl = s.parse::<f64>();
                    if fl.is_ok() {
                        Ok(Type::Float(fl.unwrap()))
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

impl Register {
    pub fn value(&self) -> usize { self.value }
}

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