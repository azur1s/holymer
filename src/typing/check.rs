use crate::parse::parser::{
    Span, Spanned,
    UnaryOp, BinaryOp, Lit, Expr,
};
use super::{ty::Type, typed::TExpr};

#[derive(Clone, Debug)]
struct TypeEnv<'src> {
    bindings: Vec<(&'src str, Type)>,
    funcs: Vec<(&'src str, Vec<Type>, Type)>,
}

impl<'src> TypeEnv<'src> {
    fn new() -> Self {
        Self {
            bindings: Vec::new(),
            funcs: Vec::new(),
        }
    }

    /// Bind a type to a name.
    fn bind(&mut self, name: &'src str, ty: Type) {
        self.bindings.push((name, ty));
    }

    /// Bind a function (parameters and return type) to a name.
    fn bind_func(&mut self, name: &'src str, args: Vec<Type>, ret_ty: Type) {
        self.funcs.push((name, args, ret_ty));
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        self.bindings.iter()
            .rev()
            .find(|(n, _)| *n == name)
            .map(|(_, t)| t.clone())
    }

    fn lookup_func(&self, name: &str) -> Option<(Vec<Type>, Type)> {
        self.funcs.iter()
            .rev()
            .find(|(n, _, _)| *n == name)
            .map(|(_, args, ret_ty)| (args.clone(), ret_ty.clone()))
    }
}

#[derive(Debug)]
pub struct TypeError {
    pub msg: String,
    pub note: Option<String>,
    pub hint: Option<(String, Span)>,
    pub loc: Span,
}

impl TypeError {
    fn new(msg: String, loc: Span) -> Self {
        Self {
            msg,
            note: None,
            hint: None,
            loc,
        }
    }

    fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    fn with_hint(mut self, hint: String, loc: Span) -> Self {
        self.hint = Some((hint, loc));
        self
    }
}

fn type_expr<'src>(
    env: &mut TypeEnv<'src>, expr: Spanned<Expr<'src>>
) -> Result<Spanned<TExpr<'src>>, TypeError> {
    macro_rules! oks { // Spanned Ok macro.
        ($e:expr $(,)?) => {
            Ok(($e, expr.1))
        };
    }
    macro_rules! unbox { // Unbox a Spanned<Box<T>> into a Spanned<T>.
        ($e:expr) => {
            (*$e.0, $e.1)
        };
    }
    macro_rules! sbox { // Box the first value of a Spanned<T>.
        ($e:expr) => {
            (Box::new($e.0), $e.1)
        };
    }

    match expr.0 {
        Expr::Lit(lit) => match lit {
            Lit::Unit    => oks!(TExpr::Lit(Lit::Unit)),
            Lit::Bool(x) => oks!(TExpr::Lit(Lit::Bool(x))),
            Lit::Num(x)  => oks!(TExpr::Lit(Lit::Num(x))),
            Lit::Str(x)  => oks!(TExpr::Lit(Lit::Str(x))),
        }

        Expr::Ident(name) => {
            let ty = env.lookup(name)
                .ok_or(TypeError::new(format!("unknown identifier `{}`", name), expr.1))?;
            oks!(TExpr::Ident(name, ty))
        }

        Expr::Unary(op, e) => {
            let te = type_expr(env, unbox!(e))?;
            let ret_ty = match op {
                UnaryOp::Neg => Type::Num,
                UnaryOp::Not => Type::Bool,
            };

            oks!(TExpr::Unary {
                op,
                expr: sbox!(te),
                ret_ty,
            })
        }

        Expr::Binary(op, lhs, rhs) => {
            let tlhs = type_expr(env, unbox!(lhs))?;
            let trhs = type_expr(env, unbox!(rhs))?;
            let ret_ty = match op {
                BinaryOp::Add
                | BinaryOp::Sub
                | BinaryOp::Mul
                | BinaryOp::Div
                | BinaryOp::Rem => Type::Num,
                BinaryOp::And
                | BinaryOp::Or => Type::Bool,
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge => Type::Bool,
            };

            oks!(TExpr::Binary {
                op,
                lhs: sbox!(tlhs),
                rhs: sbox!(trhs),
                ret_ty,
            })
        }

        Expr::Lambda(args, body) => {
            // Create a new type environment.
            let mut new_env = env.clone();

            // Bind the arguments to the new environment.
            let mut arg_tys = Vec::new();
            for (arg, maybe_ty) in args {
                let ty = match maybe_ty {
                    Some(ty) => ty,
                    None => todo!(), // TODO: infer the type of the argument after type checking the body.
                };
                arg_tys.push((arg, ty.clone()));
                new_env.bind(arg, ty);
            }

            // Type check the body.
            let tbody = type_expr(&mut new_env, unbox!(body))?;

            // Return the typed lambda expression.
            oks!(TExpr::Lambda {
                params: arg_tys,
                body: sbox!(tbody.clone()),
                ret_ty: tbody.0.ty().clone(),
            })
        }

        Expr::Call(func, cargs) => {
            // Get span of the arguments.
            let args_span = cargs.iter()
                .map(|arg| arg.1.into_range())
                .fold(None, |acc: Option<std::ops::Range<usize>>, range| match acc {
                    Some(acc) => Some(acc.start..range.end),
                    None => Some(range),
                })
                .unwrap_or(func.1.end..func.1.end+2);

            // Type check the arguments.
            let mut targs = Vec::new();
            for arg in cargs {
                let targ = type_expr(env, arg)?;
                targs.push(targ);
            }

            // Type check the function (callee).
            let tfunc = type_expr(env, unbox!(func))?;

            // Get the function type of the callee. (if any).
            if let Some((param_tys, ret_ty)) = tfunc.0.clone().as_fn() {

                // Check if the number of arguments match the number of parameters.
                if param_tys.len() != targs.len() {
                    return Err(TypeError::new(
                        format!(
                            "expected {} arguments, got {}",
                            param_tys.len(),
                            targs.len(),
                        ),
                        args_span.into(),
                    ).with_note(format!(
                        "expected {} arguments",
                        param_tys.len(),
                    )).with_hint(
                        format!(
                            "this expect arguments of type `{}`",
                            param_tys.iter().map(|ty| ty.to_string()).collect::<Vec<_>>().join(", ")
                        ),
                        func.1,
                    ));
                }

                // Check if the types of the arguments match the types of the parameters.
                for (arg, param) in targs.iter().zip(param_tys.iter()) {
                    if arg.0.ty() != param {
                        return Err(TypeError::new(
                            format!(
                                "expected argument of type `{}`, got `{}`",
                                param,
                                arg.0.ty(),
                            ),
                            arg.1,
                        ).with_note(format!(
                            "expected argument of type `{}`",
                            param,
                        )));
                    }
                }

                // Return the typed call expression.
                oks!(TExpr::Call {
                    func: sbox!(tfunc),
                    args: targs,
                    ret_ty,
                })
            } else {
                Err(TypeError::new(
                    format!("expected function, got `{}`", tfunc.0.ty()),
                    tfunc.1,
                ))
            }
        }

        Expr::If { cond, t, f } => {
            let tcond = type_expr(env, unbox!(cond))?;
            let tt = type_expr(env, unbox!(t))?;
            let tf = type_expr(env, unbox!(f))?;

            // Check if the condition is of type `bool`.
            if tcond.0.ty() != &Type::Bool {
                return Err(TypeError::new(
                    format!("expected condition of type `bool`, got `{}`", tcond.0.ty()),
                    tcond.1,
                ));
            }

            // Check if the true and false branches have the same type.
            if tt.0.ty() != tf.0.ty() {
                return Err(TypeError::new(
                    format!(
                        "expected the branches to have the same type, got `{}` and `{}`",
                        tt.0.ty(),
                        tf.0.ty(),
                    ),
                    tf.1,
                ).with_note(format!(
                    "expected this branch to be type of `{}`",
                    tt.0.ty(),
                )));
            }

            oks!(TExpr::If {
                cond: sbox!(tcond),
                br_ty: tt.0.ty().clone(),
                t: sbox!(tt),
                f: sbox!(tf),
            })
        }

        Expr::Let { bindings, body } => {
            // Create a new type environment.
            let mut new_env = env.clone();

            // Type check the bindings.
            let mut tbindings = Vec::new();
            for (name, maybe_ty, expr) in bindings {
                let ty = match maybe_ty {
                    Some(ty) => ty,
                    None => todo!(), // TODO: infer.
                };
                let texpr = type_expr(&mut new_env, unbox!(expr))?;

                // Check if the type of the binding matches the type of the expression.
                if texpr.0.ty() != &ty {
                    return Err(TypeError::new(
                        format!(
                            "expected the binding to be of type `{}`, got `{}`",
                            ty,
                            texpr.0.ty(),
                        ),
                        texpr.1,
                    ).with_note(format!(
                        "expected this binding to be of type `{}`",
                        ty,
                    )));
                }

                tbindings.push((name, ty.clone(), sbox!(texpr)));
                new_env.bind(name, ty);
            }

            // Type check the body.
            let tbody = type_expr(&mut new_env, unbox!(body))?;

            // Return the typed let expression.
            oks!(TExpr::Let {
                bindings: tbindings,
                body: sbox!(tbody),
            })
        }

        Expr::Assign(bindings) => {
            // Create a new type environment.
            let mut new_env = env.clone();

            // Type check the bindings.
            let mut tbindings = Vec::new();
            for (name, maybe_ty, expr) in bindings {
                let ty = match maybe_ty {
                    Some(ty) => ty,
                    None => todo!(), // TODO: infer.
                };
                let texpr = type_expr(&mut new_env, unbox!(expr))?;

                // Check if the type of the binding matches the type of the expression.
                if texpr.0.ty() != &ty {
                    return Err(TypeError::new(
                        format!(
                            "expected the binding to be of type `{}`, got `{}`",
                            ty,
                            texpr.0.ty(),
                        ),
                        texpr.1,
                    ).with_note(format!(
                        "expected this binding to be of type `{}`",
                        ty,
                    )));
                }

                tbindings.push((name, ty.clone(), sbox!(texpr)));
                new_env.bind(name, ty);
            }

            // Return the typed assign expression.
            oks!(TExpr::Assign(tbindings))
        }

        _ => todo!(),
    }
}

pub fn check(es: Vec<Spanned<Expr<'_>>>) -> Result<Vec<Spanned<TExpr<'_>>>, TypeError> {
    let mut env = TypeEnv::new();
    let mut tes = Vec::new();
    for e in es {
        let te = type_expr(&mut env, e)?;
        tes.push(te);
    }
    Ok(tes)
}