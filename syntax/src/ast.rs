use std::fmt::Display;

pub type Spanned<T> = (T, std::ops::Range<usize>);

#[derive(Clone, Debug)]
pub enum BuiltinType {
    Any, Null, Undefined,
    Boolean, Int, String,
}

#[derive(Clone, Debug)]
pub enum Typehint {
    Builtin(BuiltinType),
    Single(String),
    Tuple(Vec<Spanned<Self>>),
    Vector(Box<Spanned<Self>>),
    Function(Vec<Spanned<Self>>, Box<Spanned<Self>>),
    Union(Vec<Spanned<Self>>),
}

#[derive(Clone, Debug)]
pub enum Literal {
    Int(i64),
    String(String),
    Boolean(bool),
}

#[derive(Clone, Debug)]
pub enum UnaryOp { Minus, Not }

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnaryOp::Minus => write!(f, "-"),
            UnaryOp::Not   => write!(f, "!"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Plus, Minus, Multiply, Divide, Modulus,
    Equal, NotEqual, Less, Greater,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BinaryOp::Plus     => write!(f, "+"),
            BinaryOp::Minus    => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide   => write!(f, "/"),
            BinaryOp::Modulus  => write!(f, "%"),
            BinaryOp::Equal    => write!(f, "==="),
            BinaryOp::NotEqual => write!(f, "!=="),
            BinaryOp::Less     => write!(f, "<"),
            BinaryOp::Greater  => write!(f, ">"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Spanned<Literal>),
    Identifier(Spanned<String>),
    Tuple(Vec<Spanned<Self>>),
    Vector(Vec<Spanned<Self>>),
    Object {
        fields: Vec<(Spanned<String>, Spanned<Self>)>,
    },

    Unary { op: UnaryOp, rhs: Box<Spanned<Self>> },
    Binary { op: BinaryOp, lhs: Box<Spanned<Self>>, rhs: Box<Spanned<Self>> },

    Call { name: Box<Spanned<Self>>, args: Vec<Spanned<Self>> },
    Method { obj: Box<Spanned<Self>>, name: Box<Spanned<Self>>, args: Vec<Spanned<Self>> },
    Access { obj: Box<Spanned<Self>>, name: Box<Spanned<Self>> },
    Intrinsic { name: Box<Spanned<Self>>, args: Vec<Spanned<Self>> },

    Define {
        name: Spanned<String>,
        typehint: Spanned<Typehint>,
        value: Box<Spanned<Self>>
    },
    Redefine {
        name: Spanned<String>,
        value: Box<Spanned<Self>>
    },
    Function {
        name: Spanned<String>,
        generics: Vec<Spanned<String>>,
        args: Vec<(Spanned<String>, Spanned<Typehint>)>,
        typehint: Spanned<Typehint>,
        body: Box<Spanned<Self>>
    },

    If {
        cond: Box<Spanned<Self>>,
        t: Box<Spanned<Self>>,
        f: Box<Spanned<Self>>
    },
    Case {
        cond: Box<Spanned<Self>>,
        cases: Spanned<Vec<(Spanned<Self>, Spanned<Self>)>>,
        default: Box<Spanned<Self>>
    },
    Do {
        body: Spanned<Vec<Spanned<Self>>>
    },

    Return(Box<Spanned<Self>>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
}