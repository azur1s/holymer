use std::iter::Enumerate;

use nom::{InputTake, Needed, InputIter, InputLength};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Illegal, EndOfFile,

    Identifier(String), String(String),
    Int(i64), Bool(bool),

    Assign, Typehint,

    Plus, Minus, Mul, Div, Not,
    Eq, NEq, Lt, Gt, Lte, Gte,

    LParen, RParen,
    LBrace, RBrace,
    Semicolon, Colon, Comma,

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

impl<'a> InputTake for Tokens<'a> {
    #[inline]
    fn take(&self, count: usize) -> Self {
        Tokens {
            tokens: &self.tokens[0..count],
            start: 0,
            end: count,
        }
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.tokens.split_at(count);
        let first = Tokens {
            tokens: prefix,
            start: 0,
            end: prefix.len(),
        };
        let second = Tokens {
            tokens: suffix,
            start: 0,
            end: suffix.len(),
        };
        (second, first)
    }
}

impl<'a> InputLength for Tokens<'a> {
    #[inline]
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'a> InputIter for Tokens<'a> {
    type Item = &'a Token;
    type Iter = Enumerate<::std::slice::Iter<'a, Token>>;
    type IterElem = ::std::slice::Iter<'a, Token>;

    #[inline]
    fn iter_indices(&self) -> Enumerate<::std::slice::Iter<'a, Token>> {
        self.tokens.iter().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> ::std::slice::Iter<'a, Token> {
        self.tokens.iter()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where P: Fn(Self::Item) -> bool {
        self.tokens.iter().position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.tokens.len() >= count { Ok(count) }
        else { Err(Needed::Unknown) }
    }
}

pub type Program = Vec<Stmt>;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let(Ident, Ident, Expr),
    Func(Ident, Vec<Ident>, Vec<Stmt>),
    Call(Ident, Vec<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Ident(Ident), Literal(Literal),
    Array(Vec<Expr>),
    Prefix(Prefix, Box<Expr>),
    Infix(Infix, Box<Expr>, Box<Expr>),
    If {
        cond: Box<Expr>,
        then: Program,
        else_: Option<Program>,
    },
    Func {
        name: Ident,
        args: Vec<Ident>,
        body: Program,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Int(i64), Bool(bool), String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ident(pub String);

#[derive(Clone, Debug, PartialEq)]
pub enum Prefix {
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Infix {
    Plus, Minus, Mul, Div,
    Eq, NEq, Lt, Gt, Lte, Gte,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Call,
}