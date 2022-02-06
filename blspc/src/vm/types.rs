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
            Type::Cons(v) => {
                let mut s = String::new();
                s.push('(');
                for (i, t) in v.iter().enumerate() {
                    if i != 0 {  s.push(','); }
                    s.push_str(&t.fmt());
                }
                s.push(')');
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
                if s.starts_with("(") {
                    let elems = s[1..s.len() - 1]
                        .split(',')
                        .collect::<Vec<&str>>()
                        .iter()
                        .map(|s| s.trim().parse::<Type>())
                        .collect::<Result<Vec<Type>, Self::Err>>()?;
                    Ok(Type::Cons(elems))
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
