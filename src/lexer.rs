use regex::Regex;
use anyhow::{anyhow, Error};

const REGEX : &str = r###"[\s,]*([\[\]{}()]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('",;)]+)"###;

#[derive(Debug)]
pub struct Token {
    pub value: String,
    pub span: (usize, usize),
}

impl Token {
    pub fn new(value: String, span: (usize, usize)) -> Token {
        Token {
            value,
            span,
        }
    }
}

pub fn lexer(input: &str) -> Result<Vec<Token>, Error> {
    let mut results: Vec<Token> = Vec::new();
    let regex = Regex::new(REGEX).unwrap();

    for capture in regex.captures_iter(input) {
        if capture[1].starts_with(";") {
            continue;
        }

        let value = capture[1].to_string();
        let position = capture.get(0).ok_or(anyhow!("No position found"))?;
        let span = (position.start(), position.end());

        results.push(Token::new(value, span));
    }

    Ok(results)
}