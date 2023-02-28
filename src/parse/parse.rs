#![allow(clippy::type_complexity)]
use chumsky::{error, prelude::*, Stream};
use std::fmt::{Display, Formatter, Result as FmtResult};
use super::past::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Delim { Paren, Brack, Brace }

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Token {
    Num(i64), Str(String), Bool(bool), Sym(String),

    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or, Not,

    Assign, Comma, Colon, Semicolon,
    Open(Delim), Close(Delim),
    Lambda, Arrow,

    Let, Func,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Token::Num(n)  => write!(f, "{}", n),
            Token::Str(s)  => write!(f, "\"{}\"", s),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Sym(s)  => write!(f, "{}", s),

            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Mod => write!(f, "%"),
            Token::Eq  => write!(f, "=="),
            Token::Neq => write!(f, "!="),
            Token::Lt  => write!(f, "<"),
            Token::Gt  => write!(f, ">"),
            Token::Lte => write!(f, "<="),
            Token::Gte => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or  => write!(f, "||"),
            Token::Not => write!(f, "!"),

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

            Token::Let  => write!(f, "let"),
            Token::Func => write!(f, "func"),
        }
    }
}

pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let num = text::int(10)
        .map(|s: String| Token::Num(s.parse().unwrap()));

    let string = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Token::Str);

    let symbol = choice((
        just("->").to(Token::Arrow),

        just('+').to(Token::Add),
        just('-').to(Token::Sub),
        just('*').to(Token::Mul),
        just('/').to(Token::Div),
        just('%').to(Token::Mod),
        just("==").to(Token::Eq),
        just("!=").to(Token::Neq),
        just("<=").to(Token::Lte),
        just(">=").to(Token::Gte),
        just('<').to(Token::Lt),
        just('>').to(Token::Gt),
        just("&&").to(Token::And),
        just("||").to(Token::Or),
        just('!').to(Token::Not),

        just('=').to(Token::Assign),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just(';').to(Token::Semicolon),
        just('\\').to(Token::Lambda),
    ));

    let delim = choice((
        just('(').to(Token::Open(Delim::Paren)),
        just(')').to(Token::Close(Delim::Paren)),
        just('[').to(Token::Open(Delim::Brack)),
        just(']').to(Token::Close(Delim::Brack)),
        just('{').to(Token::Open(Delim::Brace)),
        just('}').to(Token::Close(Delim::Brace)),
    ));

    let kw = text::ident()
        .map(|s: String| match s.as_str() {
            "true"  => Token::Bool(true),
            "false" => Token::Bool(false),
            "let"   => Token::Let,
            "func"  => Token::Func,
            _       => Token::Sym(s),
        });

    let token = num
        .or(string)
        .or(symbol)
        .or(delim)
        .or(kw)
        .map_with_span(move |token, span| (token, span))
        .padded()
        .recover_with(skip_then_retry_until([]));

    let comments = just('/')
        .then_ignore(
            just('*')
                .ignore_then(take_until(just("*/")).ignored())
                .or(just('/').ignore_then(none_of('\n').ignored().repeated().ignored())),
        )
        .padded()
        .ignored()
        .repeated();

    token
        .padded_by(comments)
        .repeated()
        .padded()
        .then_ignore(end())
}

pub fn lex(src: String) -> (Option<Vec<(Token, Span)>>, Vec<Simple<char>>) {
    let (tokens, lex_error) = lexer().parse_recovery(src.as_str());
    (tokens, lex_error)
}

pub trait P<T> = chumsky::Parser<Token, T, Error = Simple<Token>> + Clone;

pub fn literal_parser() -> impl P<PLiteral> {
    filter_map(|span, token| match token {
        Token::Num(i)  => Ok(PLiteral::Num(i)),
        Token::Bool(b) => Ok(PLiteral::Bool(b)),
        Token::Str(s)  => Ok(PLiteral::Str(s)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    })
    .labelled("literal")
}

pub fn symbol_parser() -> impl P<String> {
    filter_map(|span, token| match token {
        Token::Sym(s) => Ok(s),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    })
    .labelled("symbol")
}

pub fn nested_parser<'a, T: 'a>(
    parser: impl P<T> + 'a,
    delim: Delim,
    f: impl Fn(Span) -> T + Clone + 'a,
) -> impl P<T> + 'a {
    parser
        .delimited_by(just(Token::Open(delim)), just(Token::Close(delim)))
        .recover_with(nested_delimiters(
            Token::Open(delim),
            Token::Close(delim),
            [
                (
                    Token::Open(Delim::Paren),
                    Token::Close(Delim::Paren),
                ),
                (
                    Token::Open(Delim::Brack),
                    Token::Close(Delim::Brack),
                ),
                (
                    Token::Open(Delim::Brace),
                    Token::Close(Delim::Brace),
                ),
            ],
            f,
        ))
        .boxed()
}
