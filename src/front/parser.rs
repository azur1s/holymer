use nom::{
    branch::alt,
    bytes::complete::take,
    combinator::{verify, map, opt},
    Err,
    error::{Error, ErrorKind},
    IResult,
    multi::many0,
    sequence::{terminated, tuple, pair, preceded, delimited}, error_position,
};

use super::model::{Token, Tokens, Precedence, Infix, Program, Stmt, Expr, Ident, Literal, Prefix};

macro_rules! tag_token (
    ($func_name:ident, $tag: expr) => (
        fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
            verify(take(1usize), |t: &Tokens| t.tokens[0] == $tag)(tokens)
        }
    )
);

tag_token!(tag_let, Token::Let);
tag_token!(tag_func, Token::Func);
tag_token!(tag_return, Token::Return);
tag_token!(tag_if, Token::If);
tag_token!(tag_else, Token::Else);

tag_token!(tag_plus, Token::Plus);
tag_token!(tag_minus, Token::Minus);
tag_token!(tag_not, Token::Not);

tag_token!(tag_assign, Token::Assign);
tag_token!(tag_typehint, Token::Typehint);
tag_token!(tag_semicolon, Token::Semicolon);
tag_token!(tag_lparen, Token::LParen);
tag_token!(tag_rparen, Token::RParen);
tag_token!(tag_lbrace, Token::LBrace);
tag_token!(tag_rbrace, Token::RBrace);
tag_token!(tag_comma, Token::Comma);
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
        parse_prefix_expr,
        parse_paren_expr,
        parse_if_expr,
    ))(input)
}
 
fn parse_paren_expr(input: Tokens) -> IResult<Tokens, Expr> {
    delimited(tag_lparen, parse_expr_lowest, tag_rparen)(input)
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

fn parse_params(input: Tokens) -> IResult<Tokens, Vec<Ident>> {
    map(
        pair(parse_ident, many0(preceded(tag_comma, parse_ident))),
        |(p, ps)| [&vec![p][..], &ps[..]].concat(),
    )(input)
}

fn empty_params(input: Tokens) -> IResult<Tokens, Vec<Ident>> { Ok((input, vec![])) }

fn parse_call_expr(input: Tokens, func_handle: Expr) -> IResult<Tokens, Expr> {
    map(
        delimited(
            tag_lparen,
            parse_exprs,
            tag_rparen,
        ),
        |e| Expr::Call { func: Box::new(func_handle.clone()), args: e },
    )(input)
}

fn parse_infix_expr(input: Tokens, left: Expr) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tokens.is_empty() { Err(Err::Error(error_position!(input, ErrorKind::Tag))) }
    else {
        let next = &t1.tokens[0];
        let (prec, op) = infix_operator(next);
        match op {
            None => Err(Err::Error(error_position!(input, ErrorKind::Tag))),
            Some(op) => {
                let (i2, right) = parse_expr_with(i1, prec)?;
                Ok((i2, Expr::Infix(op, Box::new(left), Box::new(right))))
            } 
        }
    }
}

fn parse_prefix_expr(input: Tokens) -> IResult<Tokens, Expr> {
    let (i1, t1) = alt((tag_plus, tag_minus, tag_not))(input)?;
    if t1.tokens.is_empty() { Err(Err::Error(error_position!(input, ErrorKind::Tag))) }
    else {
        let (i2, e) = parse_atom_expr(i1)?;
        match t1.tokens[0].clone() {
            Token::Plus => Ok((i2, Expr::Prefix(Prefix::Plus, Box::new(e)))),
            Token::Minus => Ok((i2, Expr::Prefix(Prefix::Minus, Box::new(e)))),
            Token::Not => Ok((i2, Expr::Prefix(Prefix::Not, Box::new(e)))),
            _ => Err(Err::Error(error_position!(input, ErrorKind::Tag))),
        }
    }
}

fn parse_expr(input: Tokens, precedence: Precedence, left: Expr) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?;

    if t1.tokens.is_empty() { Ok((i1, left)) }
    else {
        let p = infix_operator(&t1.tokens[0]);
        match p {
            (Precedence::Call, _) if precedence < Precedence::Call => {
                let (i2, left2) = parse_call_expr(input, left)?;
                parse_expr(i2, precedence, left2)
            },
            (ref peek, _) if precedence < *peek => {
                let (i2, left2) = parse_infix_expr(input, left)?;
                parse_expr(i2, precedence, left2)
            },
            _ => Ok((input, left)),
        }
    }
}

fn parse_if_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        tuple((
            tag_if,
            parse_expr_lowest,
            parse_block_stmt,
            parse_else_expr,
        )),
        |(_, cond, then, else_)| Expr::If { cond: Box::new(cond), then, else_ },
    )(input)
}

fn parse_else_expr(input: Tokens) -> IResult<Tokens, Option<Program>> {
    opt(preceded(tag_else, parse_block_stmt))(input)
}

fn parse_comma_exprs(input: Tokens) -> IResult<Tokens, Expr> {
    preceded(tag_comma, parse_expr_lowest)(input)
}

fn parse_exprs(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    map(
        pair(parse_expr_lowest, many0(parse_comma_exprs)),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}

fn parse_expr_with(input: Tokens, precedence: Precedence) -> IResult<Tokens, Expr> {
    let (i1, left) = parse_atom_expr(input)?;
    parse_expr(i1, precedence, left)
}

fn parse_expr_lowest(input: Tokens) -> IResult<Tokens, Expr> {
    parse_expr_with(input, Precedence::Lowest)
}

fn parse_return_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        delimited(
            tag_return,
            parse_expr_lowest,
            opt(tag_semicolon),
        ),
        Stmt::Return,
    )(input)
}

fn parse_call_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            parse_ident,
            tag_lparen,
            parse_exprs,
            tag_rparen,
            opt(tag_semicolon),
        )),
        |(ident, _, args, _, _)| Stmt::Call(ident, args),
    )(input)
}

fn parse_block_stmt(input: Tokens) -> IResult<Tokens, Program> {
    delimited(tag_lbrace, many0(parse_stmt), tag_rbrace)(input)
}

fn parse_func_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            tag_func,
            parse_ident,
            tag_typehint,
            tag_lparen,
            alt((parse_params, empty_params)),
            tag_rparen,
            tag_assign,
            parse_block_stmt,
            opt(tag_semicolon),
        )),
        |(_, ident, _, _, params, _, _, block, _)| Stmt::Func(ident, params, block),
    )(input)
}

fn parse_let_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    map(
        tuple((
            tag_let,
            parse_ident,
            tag_typehint,
            parse_ident,
            tag_assign,
            parse_expr_lowest,
            opt(tag_semicolon),
        )),
        |(_, ident, _, typehint, _, expr, _)| Stmt::Let(ident, typehint, expr),
    )(input)
}

fn parse_stmt(input: Tokens) -> IResult<Tokens, Stmt> {
    alt((
        parse_let_stmt,
        parse_func_stmt,
        parse_call_stmt,
        parse_return_stmt,
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