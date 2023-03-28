use super::{
    ty::Type,
    typed::TExpr,
};
use crate::{parse::{
    ptree::*,
    span::*,
}, span};

#[derive(Clone, Debug)]
struct TypeEnv {
    bindings: Vec<(String, Type)>,
    funcs: Vec<(String, Vec<Type>, Type)>,
}

impl TypeEnv {
    fn new() -> Self {
        Self {
            bindings: Vec::new(),
            funcs: Vec::new(),
        }
    }

    fn bind(&mut self, name: String, ty: Type) {
        self.bindings.push((name, ty));
    }

    fn bind_func(&mut self, name: String, args: Vec<Type>, ret_ty: Type) {
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

struct TypeError {
    msg: String,
    loc: Span,
}

fn type_expr(env: &mut TypeEnv, expr: Spanned<Expr>) -> Result<Spanned<TExpr>, TypeError> {
    match expr.value {
        Expr::Lit(lit) => match lit {
            Lit::Unit    => Ok(span!(TExpr::Lit(Lit::Unit), expr.span)),
            Lit::Bool(x) => Ok(span!(TExpr::Lit(Lit::Bool(x)), expr.span)),
            Lit::Num(x)  => Ok(span!(TExpr::Lit(Lit::Num(x)), expr.span)),
            Lit::Str(x)  => Ok(span!(TExpr::Lit(Lit::Str(x)), expr.span)),
        },

        Expr::Ident(name) => {
            let ty = env.lookup(&name)
                .ok_or(TypeError {
                    msg: format!("unknown identifier `{}`", name),
                    loc: expr.span.clone(),
                })?;
            Ok(span!(TExpr::Ident(name, ty), expr.span))
        },

        Expr::Unary(op, expr) => {
            let span = expr.span.clone();
            let texpr = type_expr(env, *expr)?;
            let ret_ty = match op {
                UnaryOp::Neg => Type::Num,
                UnaryOp::Not => Type::Bool,
            };
            Ok(span!(
                TExpr::Unary { op, expr: Box::new(texpr), ret_ty },
                span
            ))
        },

        Expr::Binary(op, lhs, rhs) => {
            let span = lhs.span.clone();
            let tlhs = type_expr(env, *lhs)?;
            let trhs = type_expr(env, *rhs)?;
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
            Ok(span!(
                TExpr::Binary { op, lhs: Box::new(tlhs), rhs: Box::new(trhs), ret_ty },
                span
            ))
        },

        Expr::Call(func, args) => {
            let span = func.span.clone();

            match func.value {
                Expr::Ident(name) => {
                    // Get the function's argument and return types
                    let (arg_tys, ret_ty) = env.lookup_func(&name)
                        .ok_or(TypeError {
                            msg: format!("unknown function `{}`", name),
                            loc: span.clone(),
                        })?;

                    // Create a typed identifier
                    let tfunc = TExpr::Ident(
                        name,
                        Type::Func(arg_tys.clone(), Box::new(ret_ty.clone()))
                    );

                    // Check that the number of arguments matches
                    if arg_tys.len() != args.len() {
                        return Err(TypeError {
                            msg: format!(
                                "expected {} arguments, got {}",
                                arg_tys.len(), args.len()
                            ),
                            loc: span,
                        });
                    }

                    // Type check the arguments
                    let mut targs = Vec::new();
                    for (arg, ty) in args.into_iter().zip(arg_tys) {
                        let targ = type_expr(env, arg)?;
                        if targ.value.ty() != &ty {
                            return Err(TypeError {
                                msg: format!(
                                    "expected argument of type `{}`, got `{}`",
                                    ty, targ.value.ty()
                                ),
                                loc: targ.span,
                            });
                        }
                        targs.push(targ);
                    }

                    Ok(span!(
                        TExpr::Call {
                            func: Box::new(span!(tfunc, span.clone())),
                            args: targs,
                            ret_ty
                        },
                        span
                    ))
                },
                Expr::Lambda(args, body) => {
                    // Create a new type environment
                    let mut new_env = env.clone();

                    // Bind the arguments to the new environment and also infer their types
                    let mut arg_tys = Vec::new();
                    for (arg, maybe_ty) in args {
                        let ty = match maybe_ty {
                            Some(ty) => ty,
                            None => todo!(), // TODO: infer the type
                        };
                        arg_tys.push((arg.clone(), ty.clone()));
                        env.bind(arg, ty);
                    }

                    // Type check the body
                    let tbody = type_expr(&mut new_env, *body)?;

                    // Return the typed lambda expression
                    Ok(span!(
                        TExpr::Lambda {
                            params: arg_tys,
                            body: Box::new(tbody.clone()),
                            ret_ty: tbody.value.ty().clone(),
                        },
                        span
                    ))
                },
                _ => todo!(),
            }
        },
        _ => todo!(),
    }
}