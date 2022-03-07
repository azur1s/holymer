use std::ops::Range;
use parser::Expr;

#[derive(Debug)]
pub enum Value { Int(i64), Boolean(bool), String(String), Ident(String) }

#[derive(Debug)]
pub enum IRKind {
    Define { name: String, type_hint: String, value: Box<Self> },
    Fun { name: String, return_type_hint: String, args: Vec<(String, String)>, body: Box<Self> },
    Call { name: String, args: Vec<Self> },
    Do { body: Box<Self> },
    If { cond: Box<Self>, body: Box<Self>, else_body: Box<Self> },
    Value { value: Value },
    Binary { op: String, left: Box<Self>, right: Box<Self> },
}

#[derive(Debug)]
pub struct IR {
    pub kind: IRKind,
    pub span: Range<usize>
}

impl IR {
    pub fn new(kind: IRKind, span: Range<usize>) -> Self {
        Self { kind, span }
    }
}

pub fn ast_to_ir(ast: Vec<(Expr, Range<usize>)>) -> Vec<IR> {
    let mut irs = Vec::new();
    for expr in ast {
        let ir_kind = expr_to_ir(&expr.0);
        let ir = IR::new(ir_kind, expr.1);
        irs.push(ir);
    }
    irs
}

pub fn expr_to_ir(expr: &Expr) -> IRKind {
    match expr {
        Expr::Let { name, type_hint, value } => {
            let value = expr_to_ir(&value.0);
            IRKind::Define { name: name.clone(), type_hint: gen_type_hint(type_hint), value: Box::new(value) }
        },

        Expr::Int(value)     => IRKind::Value { value: Value::Int(*value) },
        Expr::Boolean(value) => IRKind::Value { value: Value::Boolean(*value) },
        Expr::String(value)  => IRKind::Value { value: Value::String(value.clone()) },
        _ => { dbg!(expr); todo!() }
    }
}

fn gen_type_hint(type_hint: &str) -> String {
    match type_hint {
        "int"    => "int".to_string(),
        "bool"   => "bool".to_string(),
        "string" => "std::string".to_string(),
        _ => { dbg!(type_hint); todo!() }
    }
}