use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::{multispace0, alphanumeric1, alpha1, digit1},
    combinator::{map, map_res, recognize},
    IResult,
    multi::many0,
    sequence::{delimited, pair}, AsBytes,
};

use crate::syntax;
use super::{
    model::Token,
    helper::{Bytes, convert_vec_utf8, concat_slice_vec, str_from_bytes, str_to_from_str},
};

// Comparison
syntax! { equal_operator              , "==", Token::Eq }
syntax! { not_equal_operator          , "!=", Token::NEq }
syntax! { less_than_operator          , "<" , Token::Lt }
syntax! { greater_than_operator       , ">" , Token::Gt }
syntax! { less_than_equal_operator    , "<=", Token::Lte }
syntax! { greater_than_equal_operator , ">=", Token::Gte }

// Arithmetic
syntax! { assign_operator   , "=", Token::Assign }
syntax! { add_operator      , "+", Token::Plus }
syntax! { subtract_operator , "-", Token::Minus }
syntax! { multiply_operator , "*", Token::Mul }
syntax! { divide_operator   , "/", Token::Div }
syntax! { not_operator      , "!", Token::Not }

// Punctuations
syntax! { typehint_punctuation   , "::", Token::Typehint }
syntax! { returnhint_punctuation , "->", Token::Return }
syntax! { lparen_punctuation     , "(",  Token::LParen }
syntax! { rparen_punctuation     , ")",  Token::RParen }
syntax! { lbrace_punctuation     , "{",  Token::LBrace }
syntax! { rbrace_punctuation     , "}",  Token::RBrace }
syntax! { semicolon_punctuation  , ";",  Token::Semicolon }
syntax! { colon_punctuation      , ":",  Token::Colon }
syntax! { comma_punctuation      , ",",  Token::Comma }

// Operator & Punctuation
fn lex_operator_punctuation(input: &Bytes) -> IResult<&Bytes, Token> {
    alt((
        typehint_punctuation, returnhint_punctuation,
        lparen_punctuation, rparen_punctuation,
        lbrace_punctuation, rbrace_punctuation,
        semicolon_punctuation, colon_punctuation, comma_punctuation,
        
        equal_operator, not_equal_operator,
        less_than_operator, greater_than_operator,
        less_than_equal_operator, greater_than_equal_operator,

        assign_operator,
        add_operator, subtract_operator, multiply_operator, divide_operator,
        not_operator,

    ))(input)
}

// String
fn string_value(input: &Bytes) -> IResult<&Bytes, Vec<u8>> {
    let (i1, c1) = take(1usize)(input)?;
    match c1.as_bytes() {
        b"\"" => Ok((input, vec![])),
        b"\\" => {
            let (i2, c2) = take(1usize)(i1)?;
            string_value(i2).map(|(slice, done)| (slice, concat_slice_vec(c2, done)))
        }
        c => string_value(i1).map(|(slice, done)| (slice, concat_slice_vec(c, done)))
    }
}

fn string(input: &Bytes) -> IResult<&Bytes, String> {
    delimited(tag("\""), map_res(string_value, convert_vec_utf8), tag("\""))(input)
}

fn lex_string(input: &Bytes) -> IResult<&Bytes, Token> {
    map(string, |s| Token::String(s))(input)
}

// Reserved keywords & Identifiers
fn lex_reserved_identifier(input: &Bytes) -> IResult<&Bytes, Token> {
    map_res(
        recognize(pair(
                alt((alpha1, tag("_"))
            ),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s| {
            let c = str_from_bytes(s);
            c.map(|syntax| match syntax {
                "if" => Token::If,
                "else" => Token::Else,
                "let" => Token::Let,
                "func" => Token::Func,
                "return" => Token::Return,
                "true" => Token::Bool(true),
                "false" => Token::Bool(false),
                _ => Token::Identifier(syntax.to_string()),
             })
        },
    )(input)
}

// Integers
fn lex_integer(input: &Bytes) -> IResult<&Bytes, Token> {
    map(
        map_res(
            map_res(digit1, str_from_bytes),
            str_to_from_str,
        ),
        Token::Int,
    )(input)
}

// Illegal tokens
fn lex_illegal(input: &Bytes) -> IResult<&Bytes, Token> {
    map(take(1usize), |_| Token::Illegal)(input)
}

fn lex_comment(input: &Bytes) -> IResult<&Bytes, ()> {
    let (i1, c1) = take(2usize)(input)?;
    if c1.as_bytes() == b"//" {
        let (i2, _) = take_until("\n")(i1)?;
        let (i3, _) = take(1usize)(i2)?;
        let (i4, _) = multispace0(i3)?;
        let (i5, _) = lex_comment(i4)?;
        Ok((i5, ()))
    } else { Ok((input, ())) }
}

// Tokens
fn lex_token(input: &Bytes) -> IResult<&Bytes, Token> {
    let (i1, _) = lex_comment(input)?;
    alt((
        lex_operator_punctuation,
        lex_reserved_identifier,
        lex_string,
        lex_integer,
        lex_illegal,
    ))(i1)
}

fn lex_tokens(input: &Bytes) -> IResult<&Bytes, Vec<Token>> {
    many0(delimited(multispace0, lex_token, multispace0))(input)
}

pub struct Lexer;
impl Lexer {
    pub fn lex_tokens(input: &Bytes) -> IResult<&Bytes, Vec<Token>> {
        lex_tokens(input).map(|(slice, result)| (slice, [&result[..], &vec![Token::EndOfFile][..]].concat()))
    }
}