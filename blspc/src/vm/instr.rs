use std::{fmt::Display, str::FromStr};

use crate::vm::types::Type;

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
    Call { function: String },
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
            Instr::Comment { text }      => write!(f, "    ;               {}", text),

            Instr::Load { address }      => write!(f, "    LOAD            {}", address),
            Instr::Store { address }     => write!(f, "    STORE           {}", address),

            Instr::Call { function }     => write!(f, "    CALL            {}", function),

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