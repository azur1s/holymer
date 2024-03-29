use std::collections::HashMap;
use chumsky::span::SimpleSpan;
use syntax::{
    expr::{
        Lit, UnaryOp, BinaryOp,
        Expr,
    },
    ty::*,
};

use crate::rename::{rename_exprs, rename_type};

use super::typed::TExpr;

macro_rules! ok {
    ($e:expr) => {
        ($e, vec![])
    };
}

macro_rules! unbox {
    ($e:expr) => {
        (*$e.0, $e.1)
    };
}

#[derive(Clone, Debug)]
pub enum InferErrorKind {
    Error,
    Hint,
}

#[derive(Clone, Debug)]
pub struct InferError {
    pub title: String,
    pub labels: Vec<(String, InferErrorKind, SimpleSpan)>,
    pub span: SimpleSpan,
}

impl InferError {
    pub fn new<S: Into<String>>(title: S, span: SimpleSpan) -> Self {
        Self {
            title: title.into(),
            labels: Vec::new(),
            span,
        }
    }

    pub fn add_error<S: Into<String>>(mut self, reason: S, span: SimpleSpan) -> Self {
        self.labels.push((reason.into(), InferErrorKind::Error, span));
        self
    }

    pub fn add_hint<S: Into<String>>(mut self, reason: S, span: SimpleSpan) -> Self {
        self.labels.push((reason.into(), InferErrorKind::Hint, span));
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Constraint {
    t1: Type,
    t2: Type,
    // Where the constraint was generated, for error reporting
    span: SimpleSpan,
}

impl Constraint {
    fn new(t1: Type, t2: Type, span: SimpleSpan) -> Self {
        Self {
            t1,
            t2,
            span,
        }
    }
}

#[derive(Clone, Debug)]
struct Infer<'src> {
    env: HashMap<&'src str, Type>,
    subst: Vec<Type>,
    constraints: Vec<Constraint>,
}

impl<'src> Infer<'src> {
    fn new() -> Self {
        Infer {
            env: HashMap::new(),
            subst: Vec::new(),
            constraints: Vec::new(),
        }
    }

    /// Generate a fresh type variable
    fn fresh(&mut self) -> Type {
        let i = self.subst.len();
        self.subst.push(Type::Var(i));
        Type::Var(i)
    }

    /// Get a substitution for a type variable
    fn subst(&self, i: usize) -> Option<Type> {
        self.subst.get(i).cloned()
    }

    /// Add new constraint
    fn add_constraint(&mut self, c: Constraint) {
        self.constraints.push(c);
    }

    /// Check if a type variable occurs in a type
    fn occurs(&self, i: usize, t: Type) -> bool {
        use Type::*;
        match t {
            Unit | Bool | Int | Str => false,
            Var(j) => {
                if let Some(t) = self.subst(j) {
                    if t != Var(j) {
                        return self.occurs(i, t);
                    }
                }
                i == j
            },
            Func(args, ret) => {
                args.into_iter().any(|t| self.occurs(i, t)) || self.occurs(i, *ret)
            },
            Tuple(tys) => tys.into_iter().any(|t| self.occurs(i, t)),
            Array(ty) => self.occurs(i, *ty),
        }
    }

    /// Unify two types
    fn unify(&mut self, c: Constraint) -> Result<(), InferError> {
        macro_rules! constraint {
            ($t1:expr, $t2:expr) => {
                Constraint::new($t1, $t2, c.span)
            };
        }

        use Type::*;
        match (c.t1.clone(), c.t2.clone()) {
            // Literal types
            (Unit, Unit)
            | (Bool, Bool)
            | (Int, Int)
            | (Str, Str) => Ok(()),

            // Variable
            (Var(i), Var(j)) if i == j => Ok(()), // Same variables can be unified
            (Var(i), t2) => {
                // If the substitution is not the variable itself,
                // unify the substitution with t2
                if let Some(t) = self.subst(i) {
                    if t != Var(i) {
                        return self.unify(constraint!(t, t2));
                    }
                }
                // If the variable occurs in t2
                if self.occurs(i, t2.clone()) {
                    return Err(InferError::new("Infinite type", c.span)
                        .add_error(format!(
                            "This type contains itself: {}", rename_type(Var(i))
                        ), c.span));
                }
                // Set the substitution
                self.subst[i] = t2;
                Ok(())
            },
            (t1, Var(i)) => {
                if let Some(t) = self.subst(i) {
                    if t != Var(i) {
                        return self.unify(constraint!(t1, t));
                    }
                }
                if self.occurs(i, t1.clone()) {
                    return Err(InferError::new("Infinite type", c.span)
                        .add_error(format!(
                            "This type contains itself: {}",
                            rename_type(Var(i))
                        ), c.span));
                }
                self.subst[i] = t1;
                Ok(())
            },

            // Function
            (Func(a1, r1), Func(a2, r2)) => {
                // Check the number of arguments
                if a1.len() != a2.len() {
                    let e = InferError::new("Argument length mismatch", c.span)
                        .add_error(format!(
                            "This function is expected to take {} arguments, found {}",
                            a2.len(), a1.len()
                        ), c.span);
                    return Err(e);
                }
                // Unify the arguments
                for (a1, a2) in a1.into_iter().zip(a2.into_iter()) {
                    self.unify(constraint!(a1, a2))?;
                }
                // Unify the return types
                self.unify(constraint!(*r1, *r2))
            },

            // Tuple
            (Tuple(t1), Tuple(t2)) => {
                // Check the number of elements
                if t1.len() != t2.len() {
                    return Err(InferError::new("Tuple length mismatch", c.span)
                        .add_error(format!(
                            "Expected {} elements, found {}",
                            t1.len(), t2.len()
                        ), c.span));
                }
                // Unify the elements
                for (t1, t2) in t1.into_iter().zip(t2.into_iter()) {
                    self.unify(constraint!(t1, t2))?;
                }
                Ok(())
            },

            // Array
            (Array(t1), Array(t2)) => self.unify(constraint!(*t1, *t2)),

            // The rest will be type mismatch
            (t1, t2) => Err(InferError::new("Type mismatch", c.span)
                .add_error(format!(
                    "Expected {}, found {}",
                    rename_type(t1), rename_type(t2)
                ), c.span)),
        }
    }

    /// Solve the constraints by unifying them
    fn solve(&mut self) -> Vec<InferError> {
        let mut errors = Vec::new();
        for c in self.constraints.clone().into_iter() {
            if let Err(e) = self.unify(c) {
                errors.push(e);
            }
        }
        errors
    }

    /// Substitute the type variables with the substitutions
    fn substitute(&mut self, t: Type) -> Type {
        use Type::*;
        match t {
            // Only match any type that can contain type variables
            Var(i) => {
                if let Some(t) = self.subst(i) {
                    if t != Var(i) {
                        return self.substitute(t);
                    }
                }
                Var(i)
            },
            Func(args, ret) => {
                Func(
                    args.into_iter().map(|t| self.substitute(t)).collect(),
                    Box::new(self.substitute(*ret)),
                )
            },
            Tuple(tys) => Tuple(tys.into_iter().map(|t| self.substitute(t)).collect()),
            Array(ty) => Array(Box::new(self.substitute(*ty))),
            // The rest will be returned as is
            _ => t,
        }
    }

    /// Find a type variable in (typed) expression and substitute them
    fn substitute_texp(&mut self, e: TExpr<'src>) -> TExpr<'src> {
        use TExpr::*;
        match e {
            Lit(_) | Ident(_) => e,
            Unary { op, expr: (e, lspan), ret_ty } => {
                Unary {
                    op,
                    expr: (Box::new(self.substitute_texp(*e)), lspan),
                    ret_ty,
                }
            },
            Binary { op, lhs: (lhs, lspan), rhs: (rhs, rspan), ret_ty } => {
                let lhst = self.substitute_texp(*lhs);
                let rhst = self.substitute_texp(*rhs);
                Binary {
                    op,
                    lhs: (Box::new(lhst), lspan),
                    rhs: (Box::new(rhst), rspan),
                    ret_ty: self.substitute(ret_ty),
                }
            },
            Lambda { params, body: (body, bspan), ret_ty } => {
                let bodyt = self.substitute_texp(*body);
                let paramst = params.into_iter()
                    .map(|(name, ty)| (name, self.substitute(ty)))
                    .collect::<Vec<_>>();
                Lambda {
                    params: paramst,
                    body: (Box::new(bodyt), bspan),
                    ret_ty: self.substitute(ret_ty),
                }
            },
            Call { func: (func, fspan), args } => {
                let funct = self.substitute_texp(*func);
                let argst = args.into_iter()
                    .map(|(arg, span)| (self.substitute_texp(arg), span))
                    .collect::<Vec<_>>();
                Call {
                    func: (Box::new(funct), fspan),
                    args: argst,
                }
            },
            If { cond: (cond, cspan), t: (t, tspan), f: (f, fspan), br_ty } => {
                let condt = self.substitute_texp(*cond);
                let tt = self.substitute_texp(*t);
                let ft = self.substitute_texp(*f);
                If {
                    cond: (Box::new(condt), cspan),
                    t: (Box::new(tt), tspan),
                    f: (Box::new(ft), fspan),
                    br_ty,
                }
            },
            Let { name, ty, value: (v, vspan), body: (b, bspan) } => {
                let vt = self.substitute_texp(*v);
                let bt = self.substitute_texp(*b);
                Let {
                    name,
                    ty: self.substitute(ty),
                    value: (Box::new(vt), vspan),
                    body: (Box::new(bt), bspan),
                }
            },
            Define { name, ty, value: (v, vspan) } => {
                let vt = self.substitute_texp(*v);
                Define {
                    name,
                    ty: self.substitute(ty),
                    value: (Box::new(vt), vspan),
                }
            },
            Block { exprs, void, ret_ty } => {
                let exprst = exprs.into_iter()
                    .map(|(e, span)| (self.substitute_texp(e), span))
                    .collect::<Vec<_>>();
                Block {
                    exprs: exprst,
                    void,
                    ret_ty: self.substitute(ret_ty),
                }
            },
        }
    }

    /// Infer the type of an expression
    fn infer(
        &mut self, e: (Expr<'src>, SimpleSpan), expected: Type
    ) -> (TExpr<'src>, Vec<InferError>) {
        let span = e.1;
        macro_rules! constraint {
            ($ty:expr) => {
                self.add_constraint(Constraint::new(expected, $ty, span))
            };
        }

        match e.0 {
            // Literal values
            // Push the constraint (expected type to be the literal type) and
            // return the typed expression
            Expr::Lit(l) => match l {
                Lit::Unit => {
                    constraint!(Type::Unit);
                    ok!(TExpr::Lit(Lit::Unit))
                }
                Lit::Bool(b) => {
                    constraint!(Type::Bool);
                    ok!(TExpr::Lit(Lit::Bool(b)))
                }
                Lit::Int(i) => {
                    constraint!(Type::Int);
                    ok!(TExpr::Lit(Lit::Int(i)))
                }
                Lit::Str(s) => {
                    constraint!(Type::Str);
                    ok!(TExpr::Lit(Lit::Str(s)))
                }
            }

            // Identifiers
            // The same as literals but the type is looked up in the environment
            Expr::Ident(ref x) => {
                if let Some(t) = self.env.get(x) {
                    constraint!(t.clone());
                    ok!(TExpr::Ident(x))
                } else {
                    let kind = match &expected {
                        Type::Func(_, _) => "function",
                        _ => "value",
                    };
                    (TExpr::Ident(x), vec![
                        InferError::new(format!("Undefined {}", kind), span)
                            .add_error(format!("`{}` is not defined", x), span)
                    ])
                }
            }

            // Unary & binary operators
            // The type of the left and right hand side are inferred and
            // the expected type is determined by the operator
            Expr::Unary(op, e) => match op {
                // Numeric operators (Int -> Int)
                UnaryOp::Neg => {
                    let (te, err) = self.infer(unbox!(e), Type::Int);
                    constraint!(Type::Int);
                    (TExpr::Unary {
                        op,
                        expr: (Box::new(te), span),
                        ret_ty: Type::Int,
                    }, err)
                },
                // Boolean operators (Bool -> Bool)
                UnaryOp::Not => {
                    let (te, err) = self.infer(unbox!(e), Type::Bool);
                    constraint!(Type::Bool);
                    (TExpr::Unary {
                        op,
                        expr: (Box::new(te), span),
                        ret_ty: Type::Bool,
                    }, err)
                },
            }
            Expr::Binary(op, lhs, rhs) => match op {
                // Numeric operators (Int -> Int -> Int)
                BinaryOp::Add
                | BinaryOp::Sub
                | BinaryOp::Mul
                | BinaryOp::Div
                | BinaryOp::Rem
                => {
                    let (lt, mut errs0) = self.infer(unbox!(lhs), Type::Int);
                    let (rt, errs1) = self.infer(unbox!(rhs), Type::Int);
                    errs0.extend(errs1);
                    constraint!(Type::Int);
                    (TExpr::Binary {
                        op,
                        lhs: (Box::new(lt), lhs.1),
                        rhs: (Box::new(rt), rhs.1),
                        ret_ty: Type::Int,
                    }, errs0)
                },
                // Boolean operators (Bool -> Bool -> Bool)
                BinaryOp::And
                | BinaryOp::Or
                => {
                    let (lt, mut errs0) = self.infer(unbox!(lhs), Type::Bool);
                    let (rt, errs1) = self.infer(unbox!(rhs), Type::Bool);
                    errs0.extend(errs1);
                    constraint!(Type::Bool);
                    (TExpr::Binary {
                        op,
                        lhs: (Box::new(lt), lhs.1),
                        rhs: (Box::new(rt), rhs.1),
                        ret_ty: Type::Bool,
                    }, errs0)
                },
                // Comparison operators ('a -> 'a -> Bool)
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                => {
                    // Create a fresh type variable and then use it as the
                    // expected type for both the left and right hand side
                    // so the type on both side have to be the same
                    let t = self.fresh();
                    let (lt, mut errs0) = self.infer(unbox!(lhs), t.clone());
                    let (rt, errs1) = self.infer(unbox!(rhs), t);
                    errs0.extend(errs1);
                    constraint!(Type::Bool);
                    (TExpr::Binary {
                        op,
                        lhs: (Box::new(lt), lhs.1),
                        rhs: (Box::new(rt), rhs.1),
                        ret_ty: Type::Bool,
                    }, errs0)
                },

                BinaryOp::Pipe => {
                    // Since this is parsed with a fold left, the right hand
                    // side should always be a function
                    let t = self.fresh();
                    let (lt, mut errs0) = self.infer(unbox!(lhs), t.clone());
                    // The right hand side should be a function that takes
                    // 1 argument with the type of t
                    let (rt, errs1) = self.infer(
                        unbox!(rhs),
                        Type::Func(vec![t.clone()], Box::new(t.clone())),
                    );
                    errs0.extend(errs1);
                    constraint!(t.clone());
                    (TExpr::Binary {
                        op,
                        lhs: (Box::new(lt), lhs.1),
                        rhs: (Box::new(rt), rhs.1),
                        ret_ty: t,
                    }, errs0)
                },
            }

            // Lambda
            Expr::Lambda(args, ret, b) => {
                // Get the return type or create a fresh type variable
                let rt = ret.unwrap_or(self.fresh());
                // Fill in the type of the arguments with a fresh type
                let xs = args.into_iter()
                    .map(|(x, t)| (x, t.unwrap_or(self.fresh())))
                    .collect::<Vec<_>>();

                // Create a new environment, and add the arguments to it
                // and use the new environment to infer the body
                let mut env = self.env.clone();
                xs.clone().into_iter().for_each(|(x, t)| { env.insert(x, t); });
                let mut inf = self.clone();
                inf.env = env;
                let (bt, errs) = inf.infer(unbox!(b), rt.clone());

                // Add the substitutions & constraints from the body
                // if it doesn't already exist
                for s in inf.subst {
                    if !self.subst.contains(&s) {
                        self.subst.push(s);
                    }
                }
                for c in inf.constraints {
                    if !self.constraints.contains(&c) {
                        self.constraints.push(c);
                    }
                }

                // Push the constraints
                constraint!(Type::Func(
                    xs.clone().into_iter()
                        .map(|x| x.1)
                        .collect(),
                    Box::new(rt.clone()),
                ));

                (TExpr::Lambda {
                    params: xs,
                    body: (Box::new(bt), b.1),
                    ret_ty: rt,
                }, errs)
            },

            // Call
            Expr::Call(f, args) => {
                // Generate fresh types for the arguments
                let freshes = args.clone().into_iter()
                    .map(|_| self.fresh())
                    .collect::<Vec<Type>>();
                // Create a function type
                let fsig = Type::Func(
                    freshes.clone(),
                    Box::new(expected),
                );
                // Expect the function to have the function type
                let (ft, mut errs) = self.infer(unbox!(f), fsig);
                // Infer the arguments
                let (xs, xerrs) = args.into_iter()
                    .zip(freshes.into_iter())
                    .map(|(x, t)| {
                        let span = x.1;
                        let (xt, err) = self.infer(x, t);
                        ((xt, span), err)
                    })
                    // Flatten errors
                    .fold((vec![], vec![]), |(mut xs, mut errs), ((x, span), err)| {
                        xs.push((x, span));
                        errs.extend(err);
                        (xs, errs)
                    });
                errs.extend(xerrs);

                (TExpr::Call {
                    func: (Box::new(ft), f.1),
                    args: xs,
                }, errs)
            },

            // If
            Expr::If { cond, t, f } => {
                // Condition has to be a boolean
                let (ct, mut errs) = self.infer(unbox!(cond), Type::Bool);
                // The type of the if expression is the same as the
                // expected type
                let (tt, terrs) = self.infer(unbox!(t), expected.clone());
                let (ft, ferrs) = self.infer(unbox!(f), expected.clone());
                errs.extend(terrs);
                errs.extend(ferrs);

                (TExpr::If {
                    cond: (Box::new(ct), cond.1),
                    t: (Box::new(tt), t.1),
                    f: (Box::new(ft), f.1),
                    br_ty: expected,
                }, errs)
            },

            // Let & define
            Expr::Let { name, ty, value, body } => {
                // Infer the type of the value
                let ty = ty.unwrap_or(self.fresh());
                let (vt, mut errs) = self.infer(unbox!(value), ty.clone());

                // Create a new environment and add the binding to it
                // and then use the new environment to infer the body
                let mut env = self.env.clone();
                env.insert(name.clone(), ty.clone());
                let mut inf = Infer::new();
                inf.env = env;
                let (bt, berrs) = inf.infer(unbox!(body), expected.clone());
                errs.extend(berrs);

                for s in inf.subst {
                    if !self.subst.contains(&s) {
                        self.subst.push(s);
                    }
                }
                for c in inf.constraints {
                    if !self.constraints.contains(&c) {
                        self.constraints.push(c);
                    }
                }

                (TExpr::Let {
                    name, ty,
                    value: (Box::new(vt), value.1),
                    body: (Box::new(bt), body.1),
                }, errs)
            },
            Expr::Define { name, ty, value } => {
                let ty = ty.unwrap_or(self.fresh());
                self.env.insert(name.clone(), ty.clone());
                let (val_ty, errs) = self.infer(unbox!(value), ty.clone());

                constraint!(Type::Unit);

                (TExpr::Define {
                    name,
                    ty,
                    value: (Box::new(val_ty), value.1),
                }, errs)
            },

            // Block
            Expr::Block { exprs, void } => {
                // Infer the type of each expression
                let mut last = None;
                let len = exprs.len();
                let (texprs, errs) = exprs.into_iter()
                    .enumerate()
                    .map(|(i, x)| {
                        let span = x.1;
                        let t = self.fresh();
                        let (xt, err) = self.infer(unbox!(x), t.clone());
                        // Save the type of the last expression
                        if i == len - 1 {
                            last = Some(t);
                        }
                        ((xt, span), err)
                    })
                    .fold((vec![], vec![]), |(mut xs, mut errs), ((x, span), err)| {
                        xs.push((x, span));
                        errs.extend(err);
                        (xs, errs)
                    });

                let rt = if void || last.is_none() {
                    // If the block is void or there is no expression,
                    // the return type is unit
                    constraint!(Type::Unit);
                    Type::Unit
                } else {
                    // Otherwise, the return type is the same as the expected type
                    // constraint!(last.unwrap());
                    self.add_constraint(Constraint::new(expected.clone(), last.unwrap(), span));
                    expected
                };

                (TExpr::Block {
                    exprs: texprs,
                    void,
                    ret_ty: rt,
                }, errs)
            },
        }
    }
}

/// Infer a list of expressions
pub fn infer_exprs(es: Vec<(Expr, SimpleSpan)>) -> (Vec<(TExpr, SimpleSpan)>, Vec<InferError>) {
    let mut inf = Infer::new();
    // Type expressions
    let mut tes = vec![];
    // Unsubstituted typed expressions
    let mut errors = vec![];

    for e in es {
        let span = e.1;
        let fresh = inf.fresh();
        // Infer the types
        let (te, err) = inf.infer(e, fresh);

        // Push the expression to the list
        tes.push((te.clone(), span));

        if !err.is_empty() {
            errors.extend(err);
        }
    }

    let solve_errors = inf.solve();
    if !solve_errors.is_empty() {
        errors.extend(solve_errors);
    } else {
        // Substitute the types
        tes = tes.into_iter()
            .map(|(te, s)| (inf.substitute_texp(te), s))
            .collect();
    }

    (rename_exprs(tes), errors)
}