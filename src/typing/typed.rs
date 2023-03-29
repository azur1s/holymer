use super::ty::Type;
use crate::parse::parser::{
    BinaryOp,
    UnaryOp,
    Lit,
    Spanned,
};

type TypedBinding<'src> =
    (&'src str, Type, Spanned<Box<TExpr<'src>>>);

// Typed version of the expression.
#[derive(Clone, Debug)]
pub enum TExpr<'src> {
    Lit(Lit<'src>),
    Ident(&'src str, Type),

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
        ret_ty: Type,
    },
    If {
        cond: Spanned<Box<Self>>,
        t: Spanned<Box<Self>>,
        f: Spanned<Box<Self>>,
        br_ty: Type,
    },
    Let {
        bindings: Vec<TypedBinding<'src>>,
        body: Spanned<Box<Self>>,
    },
    Assign(Vec<TypedBinding<'src>>),
    Block {
        exprs: Vec<Spanned<Self>>,
        void: bool,
        ret_ty: Type,
    },
}

impl<'src> TExpr<'src> {
    pub fn ty(&self) -> &Type {
        match self {
            TExpr::Lit(lit) => match lit {
                Lit::Unit    => &Type::Unit,
                Lit::Bool(_) => &Type::Bool,
                Lit::Num(_)  => &Type::Num,
                Lit::Str(_)  => &Type::Str,
            },
            TExpr::Ident(_, ty)          => ty,
            TExpr::Unary { ret_ty, .. }  => ret_ty,
            TExpr::Binary { ret_ty, .. } => ret_ty,
            TExpr::Lambda { ret_ty, .. } => ret_ty,
            TExpr::Call { ret_ty, .. }   => ret_ty,
            TExpr::If { br_ty, .. }      => br_ty,
            // Get the type from the body.
            TExpr::Let { body, .. }      => body.0.ty(),
            // Assignment is always unit.
            TExpr::Assign { .. }         => &Type::Unit,
            // Get the type from the last expression in the block
            // if the expression is not ended with a semicolon.
            TExpr::Block { ret_ty, .. }  => ret_ty,
        }
    }

    pub fn as_fn(self) -> Option<(Vec<Type>, Type)> {
        match self {
            TExpr::Ident(_, Type::Func(params, ret_ty)) => Some((params, *ret_ty)),
            TExpr::Lambda { params, ret_ty, .. } => {
                let p = params.into_iter()
                    .map(|(_, ty)| ty)
                    .collect();
                Some((p, ret_ty))
            }
            _ => None,
        }
    }
}