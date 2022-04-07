
pub type Spanned<T> = (T, std::ops::Range<usize>);

#[derive(Clone, Debug)]
pub enum Typehint {
    Single(String), // e.g. `int`, `bool`, `string`
    Tuple(Vec<Spanned<Self>>), // e.g. `(int, bool)`
    Vector(Box<Spanned<Self>>), // e.g. `[int]`
    Function(Vec<Spanned<Self>>, Box<Spanned<Self>>), // e.g. `(a: int, b: bool) -> string`, `(b: int) -> [bool]`
}

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64), Float(f64), Boolean(bool),
    String(String), Identifier(String),

    Tuple(Vec<Spanned<Self>>), // Wait, its all Vec<Spanned<Self>>?
    Vector(Vec<Spanned<Self>>), // Always have been

    Unary { op: String, rhs: Box<Spanned<Self>> },
    Binary { lhs: Box<Spanned<Self>>, op: String, rhs: Box<Spanned<Self>> },
    Call { name: Box<Spanned<Self>>, args: Spanned<Vec<Spanned<Self>>> },
    Pipeline { lhs: Box<Spanned<Self>>, rhs: Box<Spanned<Self>> },
    Intrinsic { name: Box<Spanned<Self>>, args: Spanned<Vec<Spanned<Self>>> },

    Let {
        public: bool,
        name: Spanned<String>,
        type_hint: Spanned<Typehint>,
        value: Box<Spanned<Self>>,
        mutable: bool,
    },
    Fun {
        public: bool,
        name: Spanned<String>,
        type_hint: Spanned<Typehint>,
        args: Spanned<Vec<(Spanned<String>, Spanned<Typehint>)>>,
        body: Box<Spanned<Self>>
    },
    Return { expr: Box<Spanned<Self>> },

    If {
        cond: Box<Spanned<Self>>,
        body: Box<Spanned<Self>>,
        else_body: Box<Spanned<Self>>
    },
    Case {
        expr: Box<Spanned<Self>>,
        cases: Spanned<Vec<(Spanned<Self>, Spanned<Self>)>>,
        default: Box<Spanned<Self>>
    },
    Do {
        body: Spanned<Vec<Spanned<Self>>>
    },

    // Hole for positional argument(s) in piping
    Hole(usize, usize), // The usize is the span of the hole (prob should be single but whatever)
}