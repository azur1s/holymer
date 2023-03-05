#![allow(clippy::type_complexity)]
use chumsky::{prelude::*, Stream};
use std::fmt::{Display, Formatter, Result as FmtResult};
use crate::trans::ty::Type;

use crate::asts::past::*;

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

    Let, In, Func, Return, If, Then, Else,
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
            "true"   => Token::Bool(true),
            "false"  => Token::Bool(false),
            "let"    => Token::Let,
            "in"     => Token::In,
            "func"   => Token::Func,
            "return" => Token::Return,
            "if"     => Token::If,
            "then"   => Token::Then,
            "else"   => Token::Else,
            _        => Token::Sym(s),
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

pub fn type_parser() -> impl P<Type> {
    recursive(|ty| {
        let litty = symbol_parser().map(|s| match s.as_str() {
                "num"  => Type::Num,
                "str"  => Type::Str,
                "bool" => Type::Bool,
                "?"    => Type::Unknown,
                _      => Type::Sym(s),
            });

        let fun = just(Token::Open(Delim::Paren))
            .ignore_then(
                ty.clone()
                    .separated_by(just(Token::Comma))
            )
            .then_ignore(just(Token::Close(Delim::Paren)))
            .then_ignore(just(Token::Arrow))
            .then(ty)
            .map(|(args, ret)| Type::Fun(args, Box::new(ret)));

        litty
            .or(fun)
            .labelled("type")
    })
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

pub fn expr_parser() -> impl P<Spanned<PExpr>> {
    recursive(|expr: Recursive<Token, Spanned<PExpr>, Simple<Token>>| {
        let lit = literal_parser().map(PExpr::Lit);
        let sym = symbol_parser().map(PExpr::Sym);

        let vec = nested_parser(
            expr.clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .map(Some),
            Delim::Brack,
            |_| None,
        )
        .map(|xs| match xs {
            Some(xs) => PExpr::Vec(xs),
            None     => PExpr::Vec(Vec::new()),
        })
        .labelled("vector");

        // (e)
        let paren_expr = just(Token::Open(Delim::Paren))
        .ignore_then(expr.clone())
        .then_ignore(just(Token::Close(Delim::Paren)))
        .map(|e| e.0)
        .labelled("parenthesized expression");

        // \[sym : type]* -> expr
        let lam = just(Token::Lambda)
        .ignore_then(
            (
                symbol_parser()
                    .then_ignore(just(Token::Colon))
                    .then(type_parser())
            )
                .repeated()
        )
        .then_ignore(just(Token::Arrow))
        .then(expr.clone())
        .map(|(args, body)| PExpr::Lambda {
            args,
            body: Box::new(body),
        })
        .labelled("lambda");

        let let_binds = symbol_parser()
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .then_ignore(just(Token::Assign))
        .then(expr.clone())
        .map(|((sym, ty), body)| (sym, ty, body))
        .boxed()
        .labelled("let binding")
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .labelled("let bindings");

        let let_in = just(Token::Let)
        .ignore_then(let_binds.clone())
        .then_ignore(just(Token::In))
        .then(expr.clone())
        .map(|(vars, body)| PExpr::Let {
            vars,
            body: Some(Box::new(body)),
        })
        .boxed()
        .labelled("let with expression");

        let let_def = just(Token::Let)
        .ignore_then(let_binds)
        .map(|vars| PExpr::Let { vars, body: None })
        .boxed()
        .labelled("let definition");

        let block = nested_parser(
            expr.clone()
                .separated_by(just(Token::Semicolon))
                .allow_trailing(),
            Delim::Brace,
            |_| Vec::new(),
        )
        .map(PExpr::Block)
        .labelled("block");

        let ret = just(Token::Return)
        .ignore_then(expr.clone())
        .map(Box::new)
        .map(PExpr::Return)
        .labelled("return");

        let ifelse = just(Token::If)
        .ignore_then(expr.clone())
        .then_ignore(just(Token::Then))
        .then(expr.clone())
        .then_ignore(just(Token::Else))
        .then(expr.clone())
        .map(|((cond, then), f)| PExpr::If {
            cond: Box::new(cond),
            t: Box::new(then),
            f: Box::new(f),
        })
        .boxed()
        .labelled("if else");

        let atom = lit
        .or(sym)
        .or(vec)
        .or(paren_expr)
        .or(lam)
        .or(let_in)
        .or(let_def)
        .or(block)
        .or(ret)
        .or(ifelse)
        .map_with_span(|e, s| (e, s))
        .boxed()
        .labelled("atom");

        // e(e*)
        let call = atom
        .then(
            nested_parser(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .map(Some),
                Delim::Paren,
                |_| None,
            )
            .or_not(),
        )
        .map_with_span(|(f, args), s| match args {
            Some(Some(args)) => (PExpr::Call(Box::new(f), args), s),
            Some(None) => (PExpr::Error, s),
            None => f,
        });

        // op e
        let unary = choice((
            just(Token::Sub).to(PUnaryOp::Neg),
            just(Token::Not).to(PUnaryOp::Not),
        ))
        .map_with_span(|op, s| (op, s))
        .repeated()
        .then(call)
        .foldr(|op, expr| {
            let s = op.1.start()..expr.1.end();
            (PExpr::Unary(op, Box::new(expr)), s)
        })
        .boxed();

        let product = unary
        .clone()
        .then(
            choice((
                just(Token::Mul).to(PBinaryOp::Mul),
                just(Token::Div).to(PBinaryOp::Div),
                just(Token::Mod).to(PBinaryOp::Mod),
            ))
            .map_with_span(|op, s| (op, s))
            .then(unary)
            .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let s = lhs.1.start()..rhs.1.end();
            (PExpr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
        })
        .boxed();

        let sum = product
        .clone()
        .then(
            choice((
                just(Token::Add).to(PBinaryOp::Add),
                just(Token::Sub).to(PBinaryOp::Sub),
            ))
            .map_with_span(|op, s| (op, s))
            .then(product)
            .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let s = lhs.1.start()..rhs.1.end();
            (PExpr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
        })
        .boxed();

         let comparison = sum
        .clone()
        .then(
            choice((
                just(Token::Eq).to(PBinaryOp::Eq),
                just(Token::Neq).to(PBinaryOp::Neq),
                just(Token::Lt).to(PBinaryOp::Lt),
                just(Token::Lte).to(PBinaryOp::Lte),
                just(Token::Gt).to(PBinaryOp::Gt),
                just(Token::Gte).to(PBinaryOp::Gte),
            ))
            .map_with_span(|op, s| (op, s))
            .then(sum)
            .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let s = lhs.1.start()..rhs.1.end();
            (PExpr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
        })
        .boxed();

        comparison
        .clone()
        .then(
            choice((
                just(Token::And).to(PBinaryOp::And),
                just(Token::Or).to(PBinaryOp::Or),
            ))
            .map_with_span(|op, s| (op, s))
            .then(comparison)
            .repeated(),
        )
        .foldl(|lhs, (op, rhs)| {
            let s = lhs.1.start()..rhs.1.end();
            (PExpr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
        })
        .boxed()
    })
}

pub fn exprs_parser() -> impl P<Vec<Spanned<PExpr>>> {
    expr_parser()
        .then_ignore(just(Token::Semicolon))
        .repeated()
}

pub fn stmt_parser() -> impl P<Spanned<PStmt>> {
    let func = just(Token::Func)
        .ignore_then(symbol_parser())
        .then(
            nested_parser(
                symbol_parser()
                    .then_ignore(just(Token::Colon))
                    .then(type_parser())
                    .separated_by(just(Token::Comma))
                    .allow_trailing(),
                Delim::Paren,
                |_| Vec::new(),
            )
        )
        .then(type_parser())
        .then_ignore(just(Token::Assign))
        .then(expr_parser().map(Box::new))
        .map(|(((name, args), ret), body)| PStmt::Func {
            name,
            args,
            ret,
            body,
        });

    let expr = expr_parser().map(PStmt::Expr);

    func
    .or(expr)
    .map_with_span(|s, span| (s, span))
}

pub fn stmts_parser() -> impl P<Vec<Spanned<PStmt>>> {
    stmt_parser()
        .then_ignore(just(Token::Semicolon))
        .repeated()
}

pub fn parse(
    tokens: Vec<Spanned<Token>>,
    len: usize,
) -> (Option<Vec<Spanned<PStmt>>>, Vec<Simple<Token>>) {
    let (ast, parse_error) = stmts_parser()
    .then_ignore(end())
    .parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

    (ast, parse_error)
}