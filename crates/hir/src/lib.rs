use std::ops::Range;
use parser::types::{Expr, Typehint};

const INTRINSICS: [&str; 8] = [
    "write",
    "read",
    "write_file",
    "read_file",
    "emit",
    "get",
    "len",
    "throw",
];

#[derive(Debug, Clone)]
pub enum Value { Int(i64), Boolean(bool), String(String), Ident(String) }

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Ident(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IRKind {
    Value { value: Value },
    Vector { values: Vec<Self> },
    Tuple { values: Vec<Self> },

    Unary {
        op: String,
        right: Box<Self>,
        span: Range<usize>
    },
    Binary {
        op: String,
        left: Box<Self>,
        right: Box<Self>,
        span: Range<usize>
    },
    Call {
        name: String,
        args: Vec<Self>,
        span: Range<usize>
    },
    Intrinsic {
        name: String,
        args: Vec<Self>,
        span: Range<usize>
    },

    Define {
        public: bool,
        name: String,
        type_hint: String,
        value: Box<Self>,
        mutable: bool,
        span: Range<usize>,
    },
    Fun {
        public: bool,
        name: String,
        return_type_hint: String,
        args: Vec<(String, String)>,
        body: Box<Self>,
        span: Range<usize>,
    },

    If {
        cond: Box<Self>,
        body: Box<Self>,
        else_body: Box<Self>,
        span: Range<usize>,
    },
    Case {
        cond: Box<Self>,
        cases: Vec<(Box<Self>, Box<Self>)>,
        default: Box<Self>,
        span: Range<usize>,
    },
    Do { body: Vec<Self>, span: Range<usize> },

    Return { value: Box<Self>, span: Range<usize> },
    // Error { message: String, note: Option<String>, span: Range<usize> },
}

#[derive(Debug)]
pub struct IR {
    pub kind: IRKind,
    pub span: Range<usize>
}

#[derive(Debug, Clone)]
pub struct LoweringError {
    pub span: Range<usize>,
    pub message: String,
}

pub fn ast_to_ir(ast: Vec<(Expr, Range<usize>)>) -> (Vec<IR>, Vec<LoweringError>) {
    let mut irs = Vec::new();
    let mut errors = Vec::new();
    for expr in ast {
        let ir_kind = expr_to_ir(&expr.0);
        match ir_kind {
            (Some(ir), None) => {
                irs.push(IR { kind: ir, span: expr.1 });
            },
            (None, Some(err)) => {
                errors.push(err);
            },
            _ => unreachable!(),
        }
    }
    (irs, errors)
}

#[macro_export]
macro_rules! if_err_return {
    ($value:expr) => {
        if let Some(err) = $value { return (None, Some(err)); };
    };
}

#[macro_export]
macro_rules! return_ok {
    ($value:expr) => {
        return (Some($value), None)
    };
}

#[macro_export]
macro_rules! return_err {
    ($value:expr) => {
        return (None, Some($value))
    };
}

pub fn expr_to_ir(expr: &Expr) -> (Option<IRKind>, Option<LoweringError>) {
    match expr {
        Expr::Unary { op, rhs } => {
            let rhs_ir = expr_to_ir(&rhs.0);
            if_err_return!(rhs_ir.1);

            return_ok!(IRKind::Unary {
                op: op.to_string(),
                right: Box::new(rhs_ir.0.unwrap()),
                span: rhs.1.clone()
            });
        }

        Expr::Binary { lhs, op, rhs } => {
            let lhs_ir = expr_to_ir(&lhs.0);
            if_err_return!(lhs_ir.1);

            let rhs_ir = expr_to_ir(&rhs.0);
            if_err_return!(rhs_ir.1);

            return_ok!(IRKind::Binary {
                op: op.to_string(),
                left: Box::new(lhs_ir.0.unwrap()),
                right: Box::new(rhs_ir.0.unwrap()),
                span: lhs.1.start..rhs.1.end
            });
        },

        Expr::Call { name, args } => {
            let lname = match &name.0 {
                Expr::Identifier(s) => s.clone(),
                // Should never happen because the parser should have caught this
                _ => return_err!(LoweringError { span: name.1.clone(), message: "Expected identifier".to_string()})
            };
            let mut largs = Vec::new(); // `largs` stand for lowered args
            // Iterate over args
            for arg in &args.0 {
                // Lower each argument, if there is an error then return early
                let arg = expr_to_ir(&arg.0);
                if_err_return!(arg.1);
                largs.push(arg.0.unwrap());
            }

            return_ok!(IRKind::Call {
                name: lname,
                args: largs,
                span: name.1.start..args.1.end
            });
        },

        Expr::Pipeline { lhs, rhs } => {
            let lhs_ir = expr_to_ir(&lhs.0);
            if_err_return!(lhs_ir.1);

            match &rhs.0 {
                call @ Expr::Call { name, args }
                | call @ Expr::Intrinsic { name, args } => {
                    let cname = match &name.0 {
                        Expr::Identifier(s) => s.clone(),
                        // Should never happen because the parser should have caught this
                        _ => return_err!(LoweringError { span: name.1.clone(), message: "Expected identifier".to_string()})
                    };

                    // Get all the `Hole` indexes
                    let mut indexes = Vec::new();
                    for (i, arg) in args.0.iter().enumerate() {
                        if let Expr::Hole(..) = &arg.0 {
                            indexes.push(i);
                        }
                    }

                    // If there is no `Hole` in the args then return early
                    // if indexes.is_empty() {
                    //     return_err!(LoweringError {
                    //         span: rhs.1.clone(),
                    //         message: "Expected hole in piping".to_string(),
                    //     });
                    // }

                    // Remove the `Hole` from the args
                    let mut new_args = args.0.clone();
                    for index in indexes.iter().rev() {
                        new_args.remove(*index);
                    }

                    // Make a new call expression with the new args
                    let new_call = match call {
                        Expr::Call { name, args } => Expr::Call{
                            name: name.clone(),
                            args: (new_args, args.1.clone())
                        },
                        Expr::Intrinsic { name, args } => Expr::Intrinsic {
                            name: name.clone(),
                            args: (new_args, args.1.clone())
                        },
                        _ => unreachable!()
                    };
                    let new_call = expr_to_ir(&new_call);
                    if_err_return!(new_call.1);

                    // Lower all args
                    let mut largs = Vec::new();
                    for arg in &args.0 {
                        match arg.0 {
                            // If the arg is a `Hole` then replace it with the lowered IR
                            Expr::Hole(..) => {
                                largs.push(lhs_ir.0.clone().unwrap());
                            },
                            _ => {
                                let arg = expr_to_ir(&arg.0);
                                if_err_return!(arg.1);
                                largs.push(arg.0.unwrap());
                            }
                        }
                    }

                    // Match the call to the right IRKind
                    let ir_kind = match new_call.0.unwrap() {
                        IRKind::Call { .. } => IRKind::Call {
                            name: cname,
                            args: largs,
                            span: name.1.start..args.1.end
                        },
                        IRKind::Intrinsic { .. } => IRKind::Intrinsic {
                            name: cname,
                            args: largs,
                            span: name.1.start..args.1.end
                        },
                        _ => unreachable!()
                    };

                    return_ok!(ir_kind);
                },
                _ => return_err!(LoweringError {
                    span: rhs.1.clone(),
                    message: "Expected call".to_string()
                }),
            };
        },

        Expr::Let { public, name, type_hint, value, mutable } => {
            let lvalue = expr_to_ir(&value.0);
            if_err_return!(lvalue.1);

            return_ok!(IRKind::Define {
                public: *public,
                name: name.0.clone(),
                type_hint: gen_type_hint(&type_hint.0),
                value: Box::new(lvalue.0.unwrap()),
                mutable: *mutable,
                span: value.1.clone()
            });
        },

        Expr::Intrinsic { name, args } => {
            let lname = match &name.0 {
                Expr::Identifier(s) => {
                    if INTRINSICS.contains(&s.as_str()) { s.clone() }
                    else {
                        return_err!(LoweringError {
                            span: name.1.clone(),
                            message: format!("Unknown intrinsic: `{}`", s),
                        });
                    }
                }
                _ => return_err!(LoweringError { span: name.1.clone(), message: "Expected identifier".to_string()})
            };

            let mut largs = Vec::new();
            for arg in &args.0 {
                let larg = expr_to_ir(&arg.0);
                if_err_return!(larg.1);

                // Check if the args is string
                if let IRKind::Value{ .. } = larg.0.clone().unwrap() {
                    largs.push(larg.0.clone().unwrap());
                } else {
                    return_err!(LoweringError { span: arg.1.clone(), message: "Expected string".to_string()});
                }
            }

            return_ok!(IRKind::Intrinsic {
                name: lname,
                args: largs,
                span: name.1.start..args.1.end
            });
        },

        Expr::Fun { public, name, type_hint, args, body } => {
            // Iterate each argument and give it a type hint
            let largs = args.0.iter().map(|arg| (arg.0.0.clone(), gen_type_hint(&arg.1.0))).collect::<Vec<_>>();

            let lbody = expr_to_ir(&body.0);
            if_err_return!(lbody.1);

            return_ok!(IRKind::Fun {
                public: *public,
                name: name.0.clone(),
                return_type_hint: gen_type_hint(&type_hint.0),
                args: largs,
                body: Box::new(lbody.0.unwrap()),
                span: name.1.start..body.1.end
            });
        },

        Expr::Return { expr } => {
            let lexpr = expr_to_ir(&expr.0);
            if_err_return!(lexpr.1);

            return_ok!(IRKind::Return {
                value: Box::new(lexpr.0.unwrap()),
                span: expr.1.clone()
            });
        },

        Expr::Do { body } => {
            let mut lbody = Vec::new();

            for expr in &body.0 {
                let expr = expr_to_ir(&expr.0);
                if_err_return!(expr.1);
                lbody.push(expr.0.unwrap());
            };

            return_ok!(IRKind::Do {
                body: lbody,
                span: body.1.clone()
            });
        },

        Expr::If { cond, body, else_body } => {
            let lcond = expr_to_ir(&cond.0);
            if_err_return!(lcond.1);

            let lbody = expr_to_ir(&body.0);
            if_err_return!(lbody.1);

            let lelse_body = expr_to_ir(&else_body.0);
            if_err_return!(lelse_body.1);

            return_ok!(IRKind::If {
                cond: Box::new(lcond.0.unwrap()),
                body: Box::new(lbody.0.unwrap()),
                else_body: Box::new(lelse_body.0.unwrap()),
                span: cond.1.start..else_body.1.end
            });
        },

        Expr::Case { expr, cases, default } => {
            let lexpr = expr_to_ir(&expr.0);
            if_err_return!(lexpr.1);

            let mut lcases = Vec::new();
            for case in &cases.0 {
                let lcond = expr_to_ir(&case.0.0);
                if_err_return!(lcond.1);

                let lcase = expr_to_ir(&case.1.0);
                if_err_return!(lcase.1);

                lcases.push(
                    (Box::new(lcond.0.unwrap()), Box::new(lcase.0.unwrap()))
                );
            }

            let ldefault = expr_to_ir(&default.0);
            if_err_return!(ldefault.1);

            return_ok!(IRKind::Case {
                cond: Box::new(lexpr.0.unwrap()),
                cases: lcases,
                default: Box::new(ldefault.0.unwrap()),
                span: expr.1.start..default.1.end
            });
        },

        // TODO: Handle primitive types error (e.g. overflow)
        // For now it just leaves the value as is and let the target compiler handle it
        Expr::Int(value)        => return_ok!(IRKind::Value { value: Value::Int(*value) }),
        Expr::Boolean(value)    => return_ok!(IRKind::Value { value: Value::Boolean(*value) }),
        Expr::String(value)     => return_ok!(IRKind::Value { value: Value::String(value.clone()) }),
        Expr::Identifier(value) => return_ok!(IRKind::Value { value: Value::Ident(value.clone()) }),

        v @ Expr::Vector(values) | v @ Expr::Tuple(values) => {
            let mut lvalues = Vec::new();
            for value in values {
                let value = expr_to_ir(&value.0);
                if_err_return!(value.1);

                lvalues.push(value.0.unwrap());
            }
            match v {
                Expr::Vector(..) => return_ok!(IRKind::Vector { values: lvalues }),
                Expr::Tuple(..) => return_ok!(IRKind::Tuple { values: lvalues }),
                _ => unreachable!()
            }
        },

        // Probably will never happen because it is catched in parser
        Expr::Hole(start, end) => (None, Some(LoweringError {
            span: *start..*end,
            message: "Hole can only be used in piping, it is not allowed here.".to_string()
        })),
        _ => { dbg!(expr); todo!() }
    }
}

fn gen_type_hint(type_hint: &Typehint) -> String {
    match type_hint {
        Typehint::Single(t) => match t.as_str() {
            "any" => "any".to_string(),
            "int" => "number".to_string(),
            "bool" => "boolean".to_string(),
            _ => t.to_string()
        },
        Typehint::Tuple(ts) => {
            let types = ts.iter().map(|arg| gen_type_hint(&arg.0)).collect::<Vec<_>>();
            format!("[{}]", types.join(", "))
        },
        Typehint::Vector(t) => format!("{}[]", gen_type_hint(&t.0)),
        Typehint::Function(args, ret) => {
            let args_ty = args.iter().map(|arg| gen_type_hint(&arg.0)).collect::<Vec<_>>();
            let return_ty = gen_type_hint(&ret.0);
            format!(
                "({}) => {}",
                args_ty
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| format!("__{}: {}", i, arg))
                    .collect::<Vec<_>>()
                    .join(", "),
                return_ty
            )
        },
    }
}
