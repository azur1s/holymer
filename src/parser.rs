use regex::Regex;
use std::rc::Rc;

use crate::{
    lexer::Token,
    token::{Expr::{self, Null, List, Vector}, Return, Error::{self, ErrorString}}, list, vector,
};

const INT_REGEX: &str = r#"^-?[0-9]+$"#;
const STRING_REGEX: &str = r#""(?:\\.|[^\\"])*""#;

struct Reader {
    tokens: Vec<Token>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<Token>) -> Reader {
        Reader {
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
        "null" => Ok(Expr::Null),
        "true" => Ok(Expr::Bool(true)),
        "false" => Ok(Expr::Bool(false)),
        _ => {
            if int_regex.is_match(&token.value) {
                Ok(Expr::Number(token.value.parse().unwrap()))
            } else if string_regex.is_match(&token.value) {
                Ok(Expr::String(token.value[1..token.value.len() - 1].to_string()))
            } else {
                Ok(Expr::Symbol(token.value.to_string()))
            }
        }
    }
}

fn read_sequence(reader: &mut Reader, end: &str) -> Return {
    let mut sequence: Vec<Expr> = Vec::new();
    reader.next()?;
    loop {
        let token = match reader.peek() {
            Ok(token) => token,
            Err(_) => return Err(ErrorString(format!("Unexpected end of input, found '{}'", end))),
        };
        if token.value == end { break; }
        sequence.push(read_form(reader)?)
    }
    
    let _ = reader.next();

    match end {
        ")" => Ok(list!(sequence)),
        "]" => Ok(vector!(sequence)),
        _ => return Err(ErrorString(format!("Unknown sequence end value: '{}'", end))),
    }
}

fn read_form(reader: &mut Reader) -> Return {
    let token = reader.peek()?;
    match &token.value[..] {
        "(" => read_sequence(reader, ")"),
        ")" => Err(ErrorString("Unexpected ')'".to_string())),
        "[" => read_sequence(reader, "]"),
        "]" => Err(ErrorString("Unexpected ']'".to_string())),
        _ => read_atom(reader),
    }
}

pub fn parse(tokens: Vec<Token>) -> Return {
    if tokens.len() == 0 { return Ok(Null); }
    read_form(&mut Reader::new(tokens))
}