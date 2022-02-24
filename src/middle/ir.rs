use core::fmt;

use crate::front::parse::Expr;

#[derive(Debug, Clone)]
pub enum TypeHint {
    Int,
    Float, Double,
    Bool,
    String,
}

impl fmt::Display for TypeHint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeHint::Int => write!(f, "int"),
            TypeHint::Float => write!(f, "float"),
            TypeHint::Double => write!(f, "double"),
            TypeHint::Bool => write!(f, "bool"),
            TypeHint::String => write!(f, "char*"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    Ident(String),
}

#[derive(Debug, Clone)]
pub enum IR {
    Define { name: String, type_hint: TypeHint, value: Box<Self> },
    Fun { name: String, return_type_hint: TypeHint, args: Vec<(String, TypeHint)>, body: Box<Self> },
    Call { name: String, args: Vec<Self> },
    Do { body: Vec<Self> },
    If { cond: Box<Self>, body: Box<Self>, else_body: Box<Self> },
    Value { value: Value },
    Binary { op: String, left: Box<Self>, right: Box<Self> },
}

pub fn ast_to_ir(ast: &[Expr]) -> Vec<IR> {
    let mut ir = Vec::new();
    for expr in ast {
        ir.push(expr_to_ir(expr));
    }
    ir
}

pub fn expr_to_ir(expr: &Expr) -> IR {
    match expr {
        Expr::Let { name, type_hint, value } => IR::Define {
            name: name.clone(),
            type_hint: get_typehint(type_hint),
            value: Box::new(expr_to_ir(value)),
        },
        Expr::Fun { name, type_hint, args, body } => IR::Fun {
            name: name.clone(),
            return_type_hint: get_typehint(type_hint),
            args: args
                .iter()
                .map(|(name, type_hint)| (name.to_string(), get_typehint(type_hint)))
                .collect::<Vec<_>>(),
            body: Box::new(expr_to_ir(body)),
        },
        Expr::Call { name, args } => IR::Call {
            name: match &**name {
                Expr::Ident(s) => s.clone(),
                _ => panic!("Expected ident in call"),
            },
            args: args.iter().map(|arg| expr_to_ir(arg)).collect(),
        },
        Expr::Do { body } => IR::Do {
            body: body
                .iter()
                .map(|expr| expr_to_ir(expr))
                .collect::<Vec<_>>(),
        },
        Expr::Binary { op, left, right } => IR::Binary {
            op: op.to_string(),
            left: Box::new(expr_to_ir(left)),
            right: Box::new(expr_to_ir(right)),
        },
        Expr::Int(value)    => IR::Value { value: Value::Int(*value) },
        Expr::Float(value)  => IR::Value { value: Value::Double(*value) }, // TODO: Actually use float
        // Expr::Double(value) => IR::Value { value: Value::Double(*value) },
        Expr::Bool(value)   => IR::Value { value: Value::Bool(*value) },
        Expr::String(value) => IR::Value { value: Value::String(value.clone()) },
        Expr::Ident(name)   => IR::Value { value: Value::Ident(name.clone()) },
        _ => { println!("{:?}", expr); todo!() }
    }
}

fn get_typehint(from: &String) -> TypeHint {
    match from.as_str() {
        "int" => TypeHint::Int,
        "float" => TypeHint::Float,
        "double" => TypeHint::Double,
        "bool" => TypeHint::Bool,
        "string" => TypeHint::String,
        _ => panic!("Unsupported type hint: {}", from)
    }
}