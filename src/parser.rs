use regex::Regex;
use std::rc::Rc;

use crate::{
    lexer::{Token, here},
    token::{
        Type::{self, *}, Return, Error::{self, ErrorString},
    }, list, vector,
};

const INT_REGEX: &str = r#"^-?[0-9]+$"#;
const STRING_REGEX: &str = r#""(?:\\.|[^\\"])*""#;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Type),
    List(Vec<Expr>),
    Vector(Vec<Expr>),
    Identifier(String),
    Assign(String, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    If(Box<Expr>, Vec<Expr>, Vec<Expr>),
    While(Box<Expr>, Vec<Expr>),
    Call(String, Vec<Expr>),
    Function(String, Vec<String>, Vec<Expr>),
    NoOperation,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub,
    Mul, Div, Mod,
    Eq, Ne,
    Lt, Le, Gt, Ge,
}

struct Reader {
    src: String,
    tokens: Vec<Token>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<Token>, src: String) -> Reader {
        Reader {
            src,
            tokens,
            position: 0,
        }
    }
    fn next(&mut self) -> Result<&Token, Error> {
        self.position += 1;
        Ok(self.tokens.get(self.position - 1).ok_or(ErrorString("Underflow".to_string()))?)
    }
    fn peek(&mut self) -> Result<&Token, Error> {
        Ok(self.tokens.get(self.position).ok_or(ErrorString("Underflow".to_string()))?)
    }
}

fn read_atom(reader: &mut Reader) -> Return {
    let int_regex = Regex::new(INT_REGEX).unwrap();
    let string_regex = Regex::new(STRING_REGEX).unwrap();
    
    let token = reader.next()?;
    match &token.value[..] {
        "null" => Ok(Type::Null),
        "true" => Ok(Type::Bool(true)),
        "false" => Ok(Type::Bool(false)),
        _ => {
            if int_regex.is_match(&token.value) {
                Ok(Type::Number(token.value.parse().unwrap()))
            } else if string_regex.is_match(&token.value) {
                Ok(Type::Str(token.value[1..token.value.len() - 1].to_string()))
            } else {
                Ok(Type::Symbol(token.value.to_string()))
            }
        }
    }
}

fn read_sequence(reader: &mut Reader, end: &str) -> Return {
    let mut sequence: Vec<Type> = Vec::new();
    let _current_token_ = reader.next()?;
    loop {
        let token = match reader.peek() {
            Ok(token) => token,
            Err(_) => return Err(ErrorString(
                format!("{} Unexpected end of input, expected '{}'", here(&reader.src, &reader.tokens[reader.position - 1]), end)
            )),
        };
        if token.value == end { break; }
        sequence.push(read_form(reader)?)
    }
    
    let _match_token_ = reader.next()?;
    match end {
        ")" => Ok(list!(sequence)),
        "]" => Ok(vector!(sequence)),
        _ => return Err(ErrorString(format!("Unknown sequence end value: '{}'", end))),
    }
}

fn read_form(reader: &mut Reader) -> Return {
    let token = reader.peek()?;
    match &token.value[..] {
        ")" => Err(ErrorString("Unexpected ')'".to_string())),
        "(" => read_sequence(reader, ")"),
        "]" => Err(ErrorString("Unexpected ']'".to_string())),
        "[" => read_sequence(reader, "]"),
        _ => read_atom(reader),
    }
}

pub fn parse(tokens: Vec<Token>, src: &str) -> Return {
    if tokens.len() == 0 { return Ok(Null); }
    read_form(&mut Reader::new(tokens, src.to_string()))
}

pub fn translate_expr(ast: Type) -> Result<Expr, String> {
    let result: Result<Expr, String>;
    
    result = match ast {
        Type::Null => Ok(Expr::Literal(Null)),
        Type::Bool(b) => Ok(Expr::Literal(Bool(b))),
        Type::Number(n) => Ok(Expr::Literal(Number(n))),
        Type::Str(s) => Ok(Expr::Literal(Str(s))),
        Type::Symbol(s) => Ok(Expr::Identifier(s)),
        Type::List(list, _) => {
            if list.len() == 0 {
                Ok(Expr::NoOperation)
            } else {
                match &list[0] {
                    Type::Symbol(s) => {
                        match s.as_str() {
                            "def" => {
                                let value = translate_expr(list[1].clone())?;
                                Ok(Expr::Assign(s.clone(), Box::new(value)))
                            }
                            "if" => {
                                let cond = translate_expr(list[1].clone())?;
                                let then = translate_expr(list[2].clone())?;
                                let else_ = translate_expr(list[3].clone())?;
                                Ok(Expr::If(Box::new(cond), vec![then], vec![else_]))
                            }
                            "while" => {
                                let cond = translate_expr(list[1].clone())?;
                                let body = translate_expr(list[2].clone())?;
                                Ok(Expr::While(Box::new(cond), vec![body]))
                            }
                            // (fn [args] body)
                            "fun" => {
                                let function_name = match list[1].clone() {
                                    Type::Symbol(s) => s,
                                    _ => return Err(format!("Expected symbol as function name, got: {:?}", list[1]))
                                };
                                let args = match list[2].clone() {
                                    Type::Vector(v, _) => {
                                        let mut args: Vec<String> = Vec::new();
                                        for arg in v.iter() {
                                            match arg {
                                                Type::Symbol(s) => {
                                                    args.push(s.clone());
                                                }
                                                _ => return Err(format!("Unexpected type in function arguments")),
                                            }
                                        }
                                        args
                                    },
                                    _ => return Err(format!("Expected vector of args, got: {:?}", list[1])),
                                };
                                let body = translate_expr(list[3].clone())?;
                                Ok(Expr::Function(function_name, args, vec![body]))
                            }
                            "+" | "-" | "*" | "/" | "%" | "=" | "!=" | "<" | "<=" | ">" | ">=" => {
                                let left = translate_expr(list[1].clone())?;
                                let right = translate_expr(list[2].clone())?;
                                let op = match s.as_str() {
                                    "+" => BinaryOp::Add,
                                    "-" => BinaryOp::Sub,
                                    "*" => BinaryOp::Mul,
                                    "/" => BinaryOp::Div,
                                    "%" => BinaryOp::Mod,
                                    "=" => BinaryOp::Eq,
                                    "!=" => BinaryOp::Ne,
                                    "<" => BinaryOp::Lt,
                                    "<=" => BinaryOp::Le,
                                    ">" => BinaryOp::Gt,
                                    ">=" => BinaryOp::Ge,
                                    _ => return Err(format!("Unknown binary operator: '{}'", s)),
                                };
                                Ok(Expr::Binary(Box::new(left), op, Box::new(right)))
                            }
                            _ => {
                                let mut args: Vec<Expr> = Vec::new();
                                for arg in list.iter().skip(1) {
                                    args.push(translate_expr(arg.clone())?);
                                }
                                Ok(Expr::Call(s.clone(), args))
                            }
                        }
                    },
                    _ => {
                        let mut args: Vec<Expr> = Vec::new();
                        for arg in list.iter() {
                            args.push(translate_expr(arg.clone())?);
                        }
                        Ok(Expr::List(args))
                    },
                }
            }
        },
        Type::Vector(vector, _) => {
            let mut vec: Vec<Expr> = Vec::new();
            for item in vector.iter() {
                vec.push(translate_expr(item.clone())?);
            }
            return Ok(Expr::Vector(vec));
        }
    };
    
    result
}