#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Illegal, EndOfFile,

    Identifier(String), String(String),
    Int(i64), Bool(bool),

    Assign, Typehint,

    Plus, Minus, Mul, Div, Not,
    Eq, NEq, Lt, Gt, Lte, Gte,

    LParen, RParen, Semicolon, Colon,

    If, Else, Let, Func,
}

/// Token struct with position information.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens<'a> {
    pub tokens: &'a [Token],
    pub start: usize, pub end: usize,
}

impl<'a> Tokens<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Tokens { tokens, start: 0, end: tokens.len(), }
    }
}