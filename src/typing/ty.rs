use std::fmt::{self, Display, Formatter};

// TODO: Introduce lifetime here to reduce cloning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Unit, Bool, Num, Str,
    Func(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Var(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Type::Unit => write!(f, "Unit"),
            Type::Bool => write!(f, "Bool"),
            Type::Num  => write!(f, "Num"),
            Type::Str  => write!(f, "Str"),
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
            Type::Var(ref id) => write!(f, "{}", id),
        }
    }
}