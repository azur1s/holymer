use syntax::{
    expr::{
        BinaryOp,
        UnaryOp,
        Lit,
        Spanned,
    },
    ty::Type,
};

// Typed version of the expression.
#[derive(Clone, Debug)]
pub enum TExpr<'src> {
    Lit(Lit<'src>),
    Ident(&'src str),

    Unary {
        op: UnaryOp,
        expr: Spanned<Box<Self>>,
        ret_ty: Type,
    },
    Binary {
        op: BinaryOp,
        lhs: Spanned<Box<Self>>,
        rhs: Spanned<Box<Self>>,
        ret_ty: Type,
    },

    Lambda {
        params: Vec<(&'src str, Type)>,
        body: Spanned<Box<Self>>,
        ret_ty: Type,
    },
    Call {
        func: Spanned<Box<Self>>,
        args: Vec<Spanned<Self>>,
    },
    If {
        cond: Spanned<Box<Self>>,
        t: Spanned<Box<Self>>,
        f: Spanned<Box<Self>>,
        br_ty: Type,
    },
    Let {
        name: &'src str,
        ty: Type,
        value: Spanned<Box<Self>>,
        body: Spanned<Box<Self>>,
    },
    Define {
        name: &'src str,
        ty: Type,
        value: Spanned<Box<Self>>,
    },
    Block {
        exprs: Vec<Spanned<Self>>,
        void: bool,
        ret_ty: Type,
    },
}
