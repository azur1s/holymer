use std::borrow::Borrow;

use crate::front::parser::Value;
use super::instr::Instructions;

pub fn generate_instructions(ast: impl Iterator<Item = (Value, (usize, usize))>) -> Vec<Instructions> {
    let mut instructions: Vec<Instructions> = Vec::new();
    for (value, _) in ast {
        match value {
            Value::List(car, cdr) => {
                match &*car.borrow() {
                    Value::Symbol(ref function_name) => {
                        match function_name.as_str() {
                            "def" => {
                                let name: Box<str> = match &cdr[0].borrow() {
                                    Value::Symbol(name) => name.clone().into(),
                                    _ => panic!("Expected symbol as first argument of define"),
                                };

                                match &cdr[1].borrow() {
                                    Value::Int(value) => instructions.push(Instructions::Store {
                                        value: Value::Int(*value),
                                        name,
                                    }),
                                    Value::Float(value) => instructions.push(Instructions::Store {
                                        value: Value::Float(*value),
                                        name,
                                    }),
                                    Value::String(value) => instructions.push(Instructions::Store {
                                        value: Value::String(value.clone()),
                                        name,
                                    }),
                                    _ => todo!(),
                                };
                            },
                            _ => {
                                dbg!(function_name);
                                todo!();
                            }
                        } // --- End match `function_name` ---
                    },
                    _ => {
                        dbg!(car);
                        todo!();
                    }
                } // --- End match `car` ---
            }
            _ => {
                dbg!(value);
                todo!();
            }
        } // --- End match `value` ---
    }

    instructions
}