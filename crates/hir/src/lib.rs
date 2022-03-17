use std::ops::Range;
use parser::Expr;

const INTRINSICS: [&str; 4] = [
    "write",
    "read",
    "write_file",
    "emit",
];

#[derive(Debug, Clone)]
pub enum Value { Int(i64), Boolean(bool), String(String), Ident(String) }

#[derive(Debug, Clone)]
pub enum IRKind {
    Define { name: String, type_hint: String, value: Box<Self> },
    Fun { name: String, return_type_hint: String, args: Vec<(String, String)>, body: Box<Self> },
    Call { name: String, args: Vec<Self> },
    Intrinsic { name: String, args: Vec<Self> },
    Do { body: Vec<Self> },
    If { cond: Box<Self>, body: Box<Self>, else_body: Box<Self> },
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

#[derive(Debug)]
pub struct LoweringError {
    pub span: Range<usize>,
    pub message: String,
    pub note: Option<String>,
}

impl IR {
    pub fn new(kind: IRKind, span: Range<usize>) -> Self {
        Self { kind, span }
    }
}

pub fn ast_to_ir(ast: Vec<(Expr, Range<usize>)>) -> (Vec<IR>, Vec<LoweringError>) {
    let mut irs = Vec::new();
    let mut errors = Vec::new();
    for expr in ast {
        let ir_kind = expr_to_ir(&expr.0);
        if let Some(err) = ir_kind.1 {
            errors.push(err);
        } else {
            irs.push(IR::new(ir_kind.0.unwrap(), expr.1));
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

        Expr::Pipe { lhs, rhs } => {
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

                    // Get the index where the `Hole` is at
                    let index = args.0.iter().position(|arg| match arg.0 {
                        Expr::Hole(..) => true,
                        _ => false
                    });

                    // If there is no `Hole` in the args then return early
                    if let None = index {
                        return (None, Some(LoweringError {
                            span: rhs.1.clone(),
                            message: "Expected hole in piping".to_string(),
                            note: None
                        }));
                    }

                    // Remove the `Hole` from the args
                    let mut new_args = args.0.clone();
                    new_args.remove(index.unwrap());

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

        Expr::Let { name, type_hint, value } => {
            let value = expr_to_ir(&value.0);
            if_err_return!(value.1);

            let value = value.0.unwrap();
            let ir_kind = IRKind::Define { name: name.clone(), type_hint: gen_type_hint(type_hint), value: Box::new(value) };
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

        Expr::Fun { name, type_hint, args, body } => {
            // Iterate each argument and give it a type hint
            let args = args.0.iter().map(|arg| (arg.0.0.clone(), gen_type_hint(&arg.1.0))).collect::<Vec<_>>();

            let body = expr_to_ir(&body.0);
            if_err_return!(body.1);

            let body = body.0.unwrap();
            let ir_kind = IRKind::Fun { name: name.clone(), return_type_hint: gen_type_hint(type_hint), args, body: Box::new(body) };
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