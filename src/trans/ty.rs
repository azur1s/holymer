use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug)]
pub enum Type {
    Num, Str, Bool,
    Fun(Vec<Self>, Box<Self>),
    Unknown,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Type::Num => write!(f, "num"),
            Type::Str => write!(f, "str"),
            Type::Bool => write!(f, "bool"),
            Type::Fun(args, ret) => {
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> {}", ret)
            },
            Type::Unknown => write!(f, "unknown"),
        }
    }
}