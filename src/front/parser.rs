use nom::{
    bytes::complete::take,
    combinator::{verify, map},
    Err,
    IResult, sequence::{terminated, tuple}, multi::many0, branch::alt, error::{Error, ErrorKind},
};

use super::model::{Token, Tokens, Precedence, Infix, Program, Stmt, Expr, Ident, Literal};

macro_rules! tag_token (
    ($func_name:ident, $tag: expr) => (
        fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
            verify(take(1usize), |t: &Tokens| t.tokens[0] == $tag)(tokens)
        }
    )
);

tag_token!(tag_let, Token::Let);
tag_token!(tag_assign, Token::Assign);
tag_token!(tag_typehint, Token::Typehint);
tag_token!(tag_semicolon, Token::Semicolon);
tag_token!(tag_end_of_file, Token::EndOfFile);

fn infix_operator(token: &Token) -> (Precedence, Option<Infix>) {
    match *token {
        Token::Eq => (Precedence::Equals, Some(Infix::Eq)),
        Token::NEq => (Precedence::Equals, Some(Infix::NEq)),
        Token::Lt => (Precedence::LessGreater, Some(Infix::Lt)),
        Token::Gt => (Precedence::LessGreater, Some(Infix::Gt)),
        Token::Lte => (Precedence::LessGreater, Some(Infix::Lte)),
        Token::Gte => (Precedence::LessGreater, Some(Infix::Gte)),
        Token::Plus => (Precedence::Sum, Some(Infix::Plus)),
        Token::Minus => (Precedence::Sum, Some(Infix::Minus)),
        Token::Mul => (Precedence::Product, Some(Infix::Mul)),
        Token::Div => (Precedence::Product, Some(Infix::Div)),
        Token::LParen => (Precedence::Call, None),
        _ => (Precedence::Lowest, None),
    }
}

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tokens.is_empty() { Err(Err::Error(Error::new(input, ErrorKind::Tag))) }
    else {
        match t1.tokens[0].clone() {
            Token::Int(i) => Ok((i1, Literal::Int(i))),
            Token::String(s) => Ok((i1, Literal::String(s))),
            Token::Bool(b) => Ok((i1, Literal::Bool(b))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_literal_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_literal, Expr::Literal)(input)
}

fn parse_atom_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_literal_expr,
        parse_ident_expr,
    ))(input)
}
 
fn parse_ident(input: Tokens) -> IResult<Tokens, Ident> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tokens.is_empty() { Err(Err::Error(Error::new(input, ErrorKind::Tag))) }
    else {
        match t1.tokens[0].clone() {
            Token::Identifier(name) => Ok((i1, Ident(name))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_ident_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_ident, Expr::Ident)(input)
}

fn parse_let(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            tag_let,
            parse_ident,
            tag_typehint,
            parse_ident,
            tag_assign,
            parse_expr_lowest,
            tag_semicolon,
        )),
        |(_, ident, _, typehint, _, expr, _)| Stmt::Let(ident, typehint, expr),
    )(input)
}

fn parse_expr(input: Tokens, precedence: Precedence, left: Expr) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;

    if t1.tokens.is_empty() { Ok((i1, left)) }
    else {
        let p = infix_operator(&t1.tokens[0]);
        match p {
            (Precedence::Call, _) if precedence < Precedence::Call => {
                // let (i2, left2) = parse_call_expr(input, left)?;
                // parse_expr(i2, precedence, left2)
                todo!()
            },
            (ref peek, _) if precedence < *peek => {
                // let (i2, left2) = parse_infix_expr(input, left)?;
                // parse_expr(i2, precedence, left2)
                todo!()
            },
            _ => Ok((input, left)),
        }
    }
}

fn parse_expr_with(input: Tokens, precedence: Precedence) -> IResult<Tokens, Expr> {
    let (i1, left) = parse_atom_expr(input)?;
    parse_expr(i1, precedence, left)
}

fn parse_expr_lowest(input: Tokens) -> IResult<Tokens, Expr> {
    parse_expr_with(input, Precedence::Lowest)
}

fn parse_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    alt((
        parse_let,
    ))(input)
}

fn parse_program(input: Tokens) -> IResult<Tokens, Program> {
    terminated(many0(parse_stmt), tag_end_of_file)(input)
}

pub struct Parser;

impl Parser {
    pub fn parse(tokens: Tokens) -> IResult<Tokens, Program> {
        parse_program(tokens)
    }
}