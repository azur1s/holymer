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
    Cons(Vec<Type>),
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

    pub fn print(&self) -> String {
        match self {
            Type::Cons(v) => {
                let mut s = String::new();
                s.push('(');
                for (i, t) in v.iter().enumerate() {
                    if i != 0 { s.push(' '); }
                    s.push_str(&t.print().to_string());
                }
                s.push(')');
                s
            },
            Type::String(s) => s.trim().to_string(),
            _ => self.clone().to_string(),
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
            Type::Cons(v)    => {
                let mut s = String::new();
                s.push('(');
                for (i, t) in v.iter().enumerate() {
                    if i != 0 { s.push(' '); }
                    s.push_str(&t.to_string());
                }
                s.push(')');
                write!(f, "{}", s)
            },
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
                if s.starts_with("(") && s.ends_with(")") {
                    let mut v = Vec::new();
                    let mut s = s[1..s.len() - 1].to_string();
                    // TODO: This is pretty messy :(
                    while !s.is_empty() {
                        let mut i = 0;
                        while i < s.len() && s.chars().nth(i).unwrap().is_whitespace() { i += 1; }
                        s = s[i..s.len()].to_string();
                        if s.is_empty() { break; }

                        let mut j = 0;
                        while j < s.len() && !s.chars().nth(j).unwrap().is_whitespace() { j += 1; }
                        let t = &s[0..j];

                        v.push(t.parse::<Type>()?);
                        s = s[j..s.len()].to_string();
                    }
                    Ok(Type::Cons(v))
                } else {
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
}
