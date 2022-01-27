use regex::Regex;
use crate::compiler::parser::Sexpr::*;

#[derive(Debug, Clone)]
pub enum Sexpr {
    Int(i64), Float(f64), Str(String), Boolean(bool),
    Symbol(String),
    Cons(Box<Sexpr>, Vec<Sexpr>),
    Nil,
}

impl std::fmt::Display for Sexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Int(i) => write!(f, "{}", i),
            Float(fl) => write!(f, "{}", fl),
            Str(s) => write!(f, "{}", s),
            Boolean(b) => write!(f, "{}", b),
            Symbol(s) => write!(f, "{}", s),
            Cons(car, cdr) => {
                write!(f, "(")?;
                write!(f, "{}", car)?;
                for c in cdr {
                    write!(f, " {}", c)?;
                }
                write!(f, ")")
            },
            Nil => write!(f, "nil"),
        }
    }
}

pub type ParseResult = Result<Sexpr, String>;

pub struct Parser {
    unparsed: Vec<String>,
    position: usize,
}

impl Parser {
    pub fn new(src: Vec<String>) -> Parser {
        Parser {
            unparsed: src,
            position: 0,
        }
    }
    
    fn peek(&mut self) -> Option<String> {
        self.unparsed.get(self.position).cloned()
    }
    
    fn next(&mut self) -> Option<String> {
        self.position += 1;
        self.unparsed.get(self.position - 1).cloned()
    }
    
    pub fn parse(&mut self) -> ParseResult {
        match self.peek() {
            Some(s) => match s.as_str() {
                ")" => Err(format!("Unexpected ')' at position {}", self.position)),
                // TODO: Handle quote and that stuff.
                "'" => { unimplemented!() },
                "(" => self.parse_sequence(")"),
                _ => self.parse_atom(),
            }
            None => return Err("Unexpected EOF".to_string()),
        }
    }
    
    fn parse_sequence(&mut self, end: &str) -> ParseResult {
        self.next();
        let car = self.parse()?;

        let mut cdr = Vec::new();
        
        loop {
            let token = match self.peek() {
                Some(token) => token,
                None => return Err(format!("Unexpected end of input, expected '{}'", end)),
            };
            if token == end { break; }
            cdr.push(self.parse()?)
        }

        self.next();
        Ok(Sexpr::Cons(Box::new(car), cdr))
    }

    fn parse_quote_sequence(&mut self, end: &str) -> ParseResult {
        let car = Symbol("list".to_string());
        
        self.next();
        let mut cdr = Vec::new();
        loop {
            let token = match self.peek() {
                Some(token) => token,
                None => return Err(format!("Unexpected end of input, expected '{}'", end)),
            };
            if token == end { break; }
            cdr.push(self.parse()?)
        }

        self.next();
        Ok(Sexpr::Cons(Box::new(car), cdr))
    }
    
    fn parse_atom(&mut self) -> ParseResult {
        let token = self.next().unwrap();
        match token.as_str() {
            "null" => Ok(Nil),
            "true" => Ok(Boolean(true)),
            "false" => Ok(Boolean(false)),
            _ => {
                if Regex::new(r#"[+-]?([0-9]*[.])?[0-9]+"#).unwrap().is_match(&token) {
                Ok(Int(token.parse().unwrap()))
                } else if Regex::new(r#"[+-]?([0-9]*[.])?[0-9]+"#).unwrap().is_match(&token) {
                Ok(Float(token.parse().unwrap()))
                } else if Regex::new(r#""(?:\\.|[^\\"])*""#).unwrap().is_match(&token) {
                Ok(Str(token[1..token.len() - 1].to_string()))
                } else {
                    Ok(Symbol(token))
                }
            }
        }
    }
}

pub fn tokenize(str: &str) -> Vec<String> {
    let regex = Regex::new(r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"###).unwrap();
    let mut res = vec![];
    for cap in regex.captures_iter(str) {
        if cap[1].starts_with(";") { continue; }
        res.push(String::from(&cap[1]));
    }
    res
}