use std::ops::Range;
use parser::Expr;

const INTRINSICS: [&str; 5] = [
    "write",
    "read",
    "write_file",
    "read_file",
    "emit",
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
    Define {
        public: bool,
        name: String,
        type_hint: String,
        value: Box<Self>,
        mutable: bool
    },
    Fun {
        public: bool,
        name: String,
        return_type_hint: String,
        args: Vec<(String, String)>,
        body: Box<Self>
    },

    Call { name: String, args: Vec<Self> },
    Intrinsic { name: String, args: Vec<Self> },
    Do { body: Vec<Self> },
    If { cond: Box<Self>, body: Box<Self>, else_body: Box<Self> },
    Case { cond: Box<Self>, cases: Vec<(Box<Self>, Box<Self>)>, default: Box<Self> },
    Unary { op: String, right: Box<Self> },
    Binary { op: String, left: Box<Self>, right: Box<Self> },
    Value { value: Value },
    Return { value: Box<Self> },
}

#[derive(Debug)]
pub struct IR {
    pub kind: IRKind,
    pub span: Range<usize>
}

impl std::fmt::Display for IRKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IRKind::Define { ref public, ref name, ref type_hint, ref value, ref mutable } => {
                write!(f, "(let {} {} {} {} {})",
                    if *public { "export" } else { "" },
                    name,
                    type_hint,
                    value,
                    if *mutable { "mut" } else { "" },
                )
            },
            IRKind::Fun { ref public, ref name, ref return_type_hint, ref args, ref body } => {
                write!(f, "(fun{} {} :{} [{}] {})",
                    if *public { " export" } else { "" },
                    name,
                    return_type_hint,
                    args.iter().map(|(name, type_hint)| format!(":{} {}", name, type_hint)).collect::<Vec<_>>().join(" "),
                    body,
                )
            },
            IRKind::Call { ref name, ref args } => {
                write!(f, "({} {})", name, args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" "))
            },
            IRKind::Intrinsic { ref name, ref args } => {
                write!(f, "(@{} {})", name, args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" "))
            },
            IRKind::Do { ref body } => {
                write!(f, "(do {})", body.iter().map(|expr| expr.to_string()).collect::<Vec<_>>().join(" "))
            },
            IRKind::If { ref cond, ref body, ref else_body } => {
                write!(f, "(if {} {} {})", cond, body, else_body)
            },
            IRKind::Case { ref cond, ref cases, ref default } => {
                write!(f, "(case {} {} {})", cond, cases.iter().map(|(cond, body)| format!("({} {})", cond, body)).collect::<Vec<_>>().join(" "), default)
            },
            IRKind::Unary { ref op, ref right } => {
                write!(f, "({} {})", op, right)
            },
            IRKind::Binary { ref op, ref left, ref right } => {
                write!(f, "({} {} {})", op, left, right)
            },
            IRKind::Value { ref value } => {
                write!(f, "{}", value)
            },
            IRKind::Return { ref value } => {
                write!(f, "(return {})", value)
            }
            #[allow(unreachable_patterns)]
            _ => { dbg!(self); unreachable!() }
        }
    }
}

#[derive(Debug)]
pub struct LoweringError {
    pub span: Range<usize>,
    pub message: String,
    pub note: Option<String>,
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

pub fn expr_to_ir(expr: &Expr) -> (Option<IRKind>, Option<LoweringError>) {
    match expr {
        Expr::Unary { op, rhs } => {
            let rhs_ir = expr_to_ir(&rhs.0);
            if_err_return!(rhs_ir.1);

            return (Some(IRKind::Unary { op: op.to_string(), right: Box::new(rhs_ir.0.unwrap()) }), None);
        }

        Expr::Binary { lhs, op, rhs } => {
            let lhs_ir = expr_to_ir(&lhs.0);
            if_err_return!(lhs_ir.1);

            let rhs_ir = expr_to_ir(&rhs.0);
            if_err_return!(rhs_ir.1);

            return (Some(IRKind::Binary { op: op.to_string(), left: Box::new(lhs_ir.0.unwrap()), right: Box::new(rhs_ir.0.unwrap()) }), None)
        },

        Expr::Call { name, args } => {
            let name = match &name.0 {
                Expr::Identifier(s) => s.clone(),
                // Should never happen because the parser should have caught this
                _ => return (None, Some(LoweringError { span: name.1.clone(), message: "Expected identifier".to_string(), note: None }))
            };
            let mut largs = Vec::new(); // `largs` stand for lowered args
            // Iterate over args
            for arg in &args.0 {
                // Lower each argument, if there is an error then return early
                let arg = expr_to_ir(&arg.0);
                if_err_return!(arg.1);
                largs.push(arg.0.unwrap());
            }
            let ir_kind = IRKind::Call { name, args: largs };
            return (Some(ir_kind), None);
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
                        _ => return (None, Some(LoweringError { span: name.1.clone(), message: "Expected identifier".to_string(), note: None }))
                    };

                    // Get all the `Hole` indexes
                    let mut indexes = Vec::new();
                    for (i, arg) in args.0.iter().enumerate() {
                        if let Expr::Hole(..) = &arg.0 {
                            indexes.push(i);
                        }
                    }

                    // If there is no `Hole` in the args then return early
                    if indexes.is_empty() {
                        return (None, Some(LoweringError {
                            span: rhs.1.clone(),
                            message: "Expected hole in piping".to_string(),
                            note: None
                        }));
                    }

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
                        IRKind::Call { .. } => IRKind::Call { name: cname, args: largs },
                        IRKind::Intrinsic { .. } => IRKind::Intrinsic { name: cname, args: largs },
                        _ => unreachable!()
                    };

                    return (Some(ir_kind), None);
                },
                _ => return (None, Some(LoweringError {
                    span: rhs.1.clone(),
                    message: "Expected call".to_string(),
                    note: None
                })),
            };
        },

        Expr::Let { public, name, type_hint, value, mutable } => {
            let value = expr_to_ir(&value.0);
            if_err_return!(value.1);

            let value = value.0.unwrap();
            let ir_kind = IRKind::Define {
                public: *public,
                name: name.clone(),
                type_hint: gen_type_hint(type_hint),
                value: Box::new(value),
                mutable: *mutable
            };

            return (Some(ir_kind), None);
        },

        Expr::Intrinsic { name, args } => {
            let name = match &name.0 {
                Expr::Identifier(s) => {
                    if INTRINSICS.contains(&s.as_str()) { s.clone() }
                    else {
                        return (None, Some(LoweringError {
                            span: name.1.clone(),
                            message: format!("Unknown intrinsic: `{}`", s),
                            note: Some(format!("Did you mean: {}?", closet_intrinsic(s.to_string())))
                        }));
                    }
                }
                _ => return (None, Some(LoweringError { span: name.1.clone(), message: "Expected identifier".to_string(), note: None }))
            };

            let mut largs = Vec::new();
            for arg in &args.0 {
                let larg = expr_to_ir(&arg.0);
                if_err_return!(larg.1);

                // Check if the args is string
                if let IRKind::Value{ .. } = larg.0.clone().unwrap() {
                    largs.push(larg.0.clone().unwrap());
                } else {
                    return (None, Some(LoweringError { span: arg.1.clone(), message: "Expected string".to_string(), note: None }))
                }
            }

            let ir_kind = IRKind::Intrinsic { name, args: largs };
            return (Some(ir_kind), None);
        },

        Expr::Fun { public, name, type_hint, args, body } => {
            // Iterate each argument and give it a type hint
            let args = args.0.iter().map(|arg| (arg.0.0.clone(), gen_type_hint(&arg.1.0))).collect::<Vec<_>>();

            let body = expr_to_ir(&body.0);
            if_err_return!(body.1);

            let body = body.0.unwrap();
            let ir_kind = IRKind::Fun {
                public: *public,
                name: name.clone(),
                return_type_hint: gen_type_hint(type_hint),
                args,
                body: Box::new(body)
            };
            return (Some(ir_kind), None);
        },

        Expr::Return { expr } => {
            let expr = expr_to_ir(&expr.0);
            if_err_return!(expr.1);

            let expr = expr.0.unwrap();
            let ir_kind = IRKind::Return { value: Box::new(expr) };
            return (Some(ir_kind), None);
        },

        Expr::Do { body } => {
            let mut lbody = Vec::new();
            for expr in body {
                let expr = expr_to_ir(&expr.0);
                if_err_return!(expr.1);
                lbody.push(expr.0.unwrap());
            }
            let ir_kind = IRKind::Do { body: lbody };
            return (Some(ir_kind), None);
        },

        Expr::If { cond, body, else_body } => {
            let cond = expr_to_ir(&cond.0);
            if_err_return!(cond.1);

            let body = expr_to_ir(&body.0);
            if_err_return!(body.1);

            let else_body = expr_to_ir(&else_body.0);
            if_err_return!(else_body.1);

            let ir_kind = IRKind::If {
                cond: Box::new(cond.0.unwrap()),
                body: Box::new(body.0.unwrap()),
                else_body: Box::new(else_body.0.unwrap())
            };
            return (Some(ir_kind), None);
        },

        Expr::Case { expr, cases, default } => {
            let expr = expr_to_ir(&expr.0);
            if_err_return!(expr.1);

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

            let default = expr_to_ir(&default.0);
            if_err_return!(default.1);

            let ir_kind = IRKind::Case {
                cond: Box::new(expr.0.unwrap()),
                cases: lcases,
                default: Box::new(default.0.unwrap())
            };
            return (Some(ir_kind), None);
        },

        // TODO: Handle primitive types error (e.g. overflow)
        // For now it just leaves the value as is and let the target compiler handle it
        Expr::Int(value)        => (Some(IRKind::Value { value: Value::Int(*value) }), None),
        Expr::Boolean(value)    => (Some(IRKind::Value { value: Value::Boolean(*value) }), None),
        Expr::String(value)     => (Some(IRKind::Value { value: Value::String(value.clone()) }), None),
        Expr::Identifier(value) => (Some(IRKind::Value { value: Value::Ident(value.clone()) }), None),
        // Probably will never happen because it is catched in parser
        Expr::Hole(start, end) => (None, Some(LoweringError {
            span: *start..*end,
            message: "Hole can only be used in piping, it is not allowed here.".to_string(),
            note: None
        })),
        _ => { dbg!(expr); todo!() }
    }
}

fn gen_type_hint(type_hint: &str) -> String {
    match type_hint {
        "int"    => "number".to_string(),
        "bool"   => "boolean".to_string(),
        "string" => "string".to_string(),
        "void"   => "void".to_string(),
        _ => { dbg!(type_hint); todo!() }
    }
}

// Get the closet intrinsic name to the given name
fn closet_intrinsic(got: String) -> String {
    let mut closest = String::new();
    let mut closest_dist = std::usize::MAX;
    for intrinsic in INTRINSICS.iter() {
        let dist = levenshtein::levenshtein(got.as_str(), intrinsic);
        if dist < closest_dist {
            closest = intrinsic.to_string();
            closest_dist = dist;
        }
    }
    closest
}
