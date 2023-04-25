use std::fmt::{self, Display, Formatter};

// TODO: Introduce lifetime here to reduce cloning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Unit, Bool, Int, Str,
    Var(usize), // This type is only used during type inference.
    Func(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Type::Unit => write!(f, "Unit"),
            Type::Bool => write!(f, "Bool"),
            Type::Int  => write!(f, "Int"),
            Type::Str  => write!(f, "Str"),
            Type::Var(id) => write!(f, "{}", itoa(id)),
            Type::Func(ref args, ref ret) => {
                write!(f, "({}", args[0])?;
                for arg in &args[1..] {
                    write!(f, " {}", arg)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Tuple(ref tys) => {
                write!(f, "({}", tys[0])?;
                for ty in &tys[1..] {
                    write!(f, " {}", ty)?;
                }
                write!(f, ")")
            }
            Type::Array(ref ty) => write!(f, "[{}]", ty),
        }
    }
}

/// Convert a number to a string of lowercase letters
pub fn itoa(i: usize) -> String {
    let mut s = String::new();
    let mut i = i;

    while i >= 26 {
        s.push((b'a' + (i % 26) as u8) as char);
        i /= 26;
    }
    s.push((b'a' + i as u8) as char);
    s
}