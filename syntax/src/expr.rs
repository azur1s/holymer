use std::fmt::{ Display, Formatter, self };
use chumsky::span::SimpleSpan;

use super::ty::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum Delim { Paren, Brack, Brace }

// The tokens of the language.
// 'src is the lifetime of the source code string.
#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Unit, Bool(bool), Int(i64), Str(&'src str),
    Ident(&'src str),

    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or, Not,
    Pipe,

    Assign, Comma, Colon, Semicolon,
    Open(Delim), Close(Delim),
    Lambda, Arrow,

    Let, In, Func, Return, If, Then, Else,
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Unit    => write!(f, "()"),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Int(n)  => write!(f, "{}", n),
            Token::Str(s)  => write!(f, "\"{}\"", s),
            Token::Ident(s)  => write!(f, "{}", s),

            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Rem => write!(f, "%"),
            Token::Eq  => write!(f, "=="),
            Token::Ne => write!(f, "!="),
            Token::Lt  => write!(f, "<"),
            Token::Gt  => write!(f, ">"),
            Token::Le => write!(f, "<="),
            Token::Ge => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or  => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Pipe => write!(f, "|>"),

            Token::Assign    => write!(f, "="),
            Token::Comma     => write!(f, ","),
            Token::Colon     => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Open(d) => write!(f, "{}", match d {
                Delim::Paren => "(",
                Delim::Brack => "[",
                Delim::Brace => "{",
            }),
            Token::Close(d) => write!(f, "{}", match d {
                Delim::Paren => ")",
                Delim::Brack => "]",
                Delim::Brace => "}",
            }),
            Token::Lambda => write!(f, "\\"),
            Token::Arrow  => write!(f, "->"),

            Token::Let    => write!(f, "let"),
            Token::In     => write!(f, "in"),
            Token::Func   => write!(f, "func"),
            Token::Return => write!(f, "return"),
            Token::If     => write!(f, "if"),
            Token::Then   => write!(f, "then"),
            Token::Else   => write!(f, "else"),
        }
    }
}

pub type Span = SimpleSpan<usize>;

#[derive(Clone, Debug, PartialEq)]
pub enum Lit<'src> {
    Unit,
    Bool(bool),
    Int(i64),
    Str(&'src str),
}

#[derive(Clone, Debug)]
pub enum UnaryOp { Neg, Not }

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Rem,
    And, Or,
    Eq, Ne, Lt, Le, Gt, Ge,
    Pipe,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Rem => write!(f, "%"),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or  => write!(f, "||"),
            BinaryOp::Eq  => write!(f, "=="),
            BinaryOp::Ne  => write!(f, "!="),
            BinaryOp::Lt  => write!(f, "<"),
            BinaryOp::Le  => write!(f, "<="),
            BinaryOp::Gt  => write!(f, ">"),
            BinaryOp::Ge  => write!(f, ">="),
            BinaryOp::Pipe => write!(f, "|>"),
        }
    }
}

pub type Spanned<T> = (T, Span);

// Clone is needed for type checking since the type checking
// algorithm is recursive and sometimes consume the AST.
#[derive(Clone, Debug)]
pub enum Expr<'src> {
    Lit(Lit<'src>),
    Ident(&'src str),

    Unary(UnaryOp, Spanned<Box<Self>>),
    Binary(BinaryOp, Spanned<Box<Self>>, Spanned<Box<Self>>),

    Lambda(Vec<(&'src str, Option<Type>)>, Option<Type>, Spanned<Box<Self>>),
    Call(Spanned<Box<Self>>, Vec<Spanned<Self>>),

    If {
        cond: Spanned<Box<Self>>,
        t: Spanned<Box<Self>>,
        f: Spanned<Box<Self>>,
    },
    Let {
        name: &'src str,
        ty: Option<Type>,
        value: Spanned<Box<Self>>,
        body: Spanned<Box<Self>>,
    },
    Define {
        name: &'src str,
        ty: Option<Type>,
        value: Spanned<Box<Self>>,
    },
    Block {
        exprs: Vec<Spanned<Box<Self>>>,
        void: bool, // True if last expression is discarded (ends with semicolon).
    },
}