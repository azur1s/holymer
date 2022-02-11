use std::{rc::Rc, borrow::Borrow};

use crate::front::parser::Value;

#[derive(Debug, Clone)]
pub enum HIRLiteral {
    True, False, Nil,
    Int(i64), Float(f64),
    String(String), Symbol(String),
    List(Box<HIRLiteral>, Vec<HIRLiteral>),
}

#[derive(Debug, Clone)]
pub enum HIR {
    Declare { name: String, value: HIRLiteral },
    Set { name: String, value: HIRLiteral },
    Let { bindings: Vec<(String, HIR)>, body: Vec<HIR> },
    If { condition: Box<HIR>, then: Box<HIR>, else_: Option<Box<HIR>> },
    Call { func: String, args: Rc<Vec<HIR>> },
    
    Quoted { body: HIRLiteral },
    Literal(HIRLiteral),
}

pub fn to_hirs(ast: &Vec<Value>) -> Vec<HIR> {
    let mut hir = Vec::new();
    for node in ast {
        match node {
            Value::List(car, cdr) => {
                match &*car.borrow() {
                    Value::Symbol(ref function) => {
                        match function.as_str() {

                            "quote" => {
                                hir.push(HIR::Quoted { body: to_hir_literal(&cdr[0].clone()) });
                            },

                            "if" => {
                                let cond = to_hir_single(&cdr[0].clone());
                                let then = to_hir_single(&cdr[1].clone());
                                let else_ = if cdr.len() > 2 { Some(Box::new(to_hir_single(&cdr[2].clone()))) }
                                else { None };

                                hir.push(HIR::If { condition: Box::new(cond), then: Box::new(then), else_ });
                            }

                            "def" => {
                                let name: String = match &cdr[0].clone() {
                                    Value::Symbol(name) => name.clone(),
                                    _ => panic!("def expects a symbol as first argument"),
                                };
                                let value = &cdr[1].clone();

                                hir.push(HIR::Declare { name, value: to_hir_literal(value) });
                            },

                            "print" => {
                                let value = &cdr[0].clone();

                                hir.push(HIR::Call { func: "print".to_string(), args: Rc::new(vec![to_hir_single(value)]) });
                            },

                            "equal" => {
                                let left = &cdr[0].clone();
                                let right = &cdr[1].clone();

                                hir.push(HIR::Call { func: "equal".to_string(), args: Rc::new(vec![to_hir_single(left), to_hir_single(right)]) });
                            },

                            _ => {
                                dbg!(function);
                                todo!();
                            }
                        } // --- End match `function` ---
                    },
                    _ => {
                        dbg!(car);
                        todo!();
                    } // --- End match `car` ---
                }
            },
            _ => hir.push(to_hir_single(node)),
        } // --- End match `node` ---
    }
    hir
}

fn to_hir_single(value: &Value) -> HIR {
    match value {
        Value::List(car, cdr) => {
            let mut vec: Vec<Value> = Vec::new();
            let list: Value = Value::List(car.clone(), cdr.clone());
            vec.push(list);
            let result = to_hirs(&vec)[0].clone();
            result
        },
        _ => HIR::Literal(to_hir_literal(value)),
    }
}

fn to_hir_literal(value: &Value) -> HIRLiteral {
    match value {
        Value::True => HIRLiteral::True,
        Value::False => HIRLiteral::False,
        Value::Int(i) => HIRLiteral::Int(*i),
        Value::Float(fl) => HIRLiteral::Float(*fl),
        Value::String(s) => HIRLiteral::String(s.clone()),
        Value::Symbol(s) => HIRLiteral::Symbol(s.clone()),
        Value::List(car, cdr) => {
            let car_literal = to_hir_literal(&car);
            let cdr_literal = cdr.iter().map(|a| to_hir_literal(a)).collect::<Vec<HIRLiteral>>();
            HIRLiteral::List(Box::new(car_literal), cdr_literal)
        },
        Value::Nil => HIRLiteral::Nil,
    }
}