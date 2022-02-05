use std::{fmt::Display, str::FromStr, ops::{Add, Sub, Mul, Div, Not}};

use crate::vm::vm::Error::{self, InvalidAriphmeticOperation};

/// Literal types for the assembler.
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Null,
    Int(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl Type {
    pub fn is_null(&self) -> bool {
        match self {
            Type::Null => true,
            _ => false,
        }
    }

    pub fn trim(&self) -> Type {
        match self {
            Type::String(s) => Type::String(s[1..s.len() - 1].to_string()),
            _ => self.clone(),
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

impl Not for Type {
    type Output = Result<Type, Error>;

    fn not(self) -> Result<Type, Error> {
        match self {
            Type::Boolean(b) => Ok(Type::Boolean(!b)),
            _ => Err(InvalidAriphmeticOperation),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Int(i)     => write!(f, "{}", i),
            Type::Float(fl)  => write!(f, "{}", fl),
            Type::Boolean(b) => write!(f, "{}", b),
            Type::String(s)  => write!(f, "\"{}\"", s),
            _ => unreachable!(),
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
        write!(f, "%{}", self.value)
    }
}

impl FromStr for Register {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("%") { return Err(format!("Invalid register: {}", s)); }

        let value = s[1..].parse::<usize>().map_err(|_| (format!("Invalid register: {}", s)))?;
        Ok(Register { value })
    }
}

/// Instructions for the assembler.
#[derive(Clone, Debug)]
pub enum Instr {
    Label { name: String }, Comment { text: String },
    // Variable declaration
    Load { address: Register }, Store { address: Register },
    // Call intrinsic function.
    Call,
    // Stack operations.
    Push { value: Type }, Pop { address: Register }, Swap,
    // Stack arithmetic operations.
    Add, Sub,
    Mul, Div,
    Not,
    // Jumping.
    JumpLabel { to: String }, // Jump to (function) label.
    Jump { to: isize }, // Jump with offset.
    JumpIfFalse { to: isize },
    // Comparison (with stack values).
    Equal,

    Return,
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            //                                        --4-- Padding
            //                                        ----------20--------- Parameter start
            Instr::Label { name }        => write!(f, ".{}:", name),
            Instr::Comment { text }      => write!(f, ";{}", text),

            Instr::Load { address }      => write!(f, "    LOAD            {}", address),
            Instr::Store { address }     => write!(f, "    STORE           {}", address),

            Instr::Call                  => write!(f, "    CALL"),

            Instr::Push { value }        => write!(f, "    PUSH            {}", value),
            Instr::Pop { address }       => write!(f, "    POP             {}", address),
            Instr::Swap                  => write!(f, "    SWAP"),

            Instr::Add                   => write!(f, "    ADD"),
            Instr::Sub                   => write!(f, "    SUB"),
            Instr::Mul                   => write!(f, "    MUL"),
            Instr::Div                   => write!(f, "    DIV"),

            Instr::Not                   => write!(f, "    NOT"),

            Instr::JumpLabel { to }      => write!(f, "    JMPL            {}", to),
            Instr::Jump { to }           => write!(f, "    JMP             {}", to),
            Instr::JumpIfFalse { to }    => write!(f, "    JMPF            {}", to),

            Instr::Equal                 => write!(f, "    EQ"),

            Instr::Return                => write!(f, "    RET"),
        }
    }
}