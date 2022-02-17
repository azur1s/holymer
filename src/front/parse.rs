use chumsky::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    // Types
    Int(i64), Float(String),
    Boolean(bool), String(String),
    Ident(String),

    // Symbols
    Operator(String),
    Delimiter(char),
    Semicolon,
    Assign, Colon,
    Comma,

    // Keywords
    Let, Fun,
}

pub type Span = std::ops::Range<usize>;
pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let int = text::int(10)
        .map(|s: String| Token::Int(s.parse().unwrap()));

    let float = text::int(10)
        .then_ignore(just('.'))
        .chain::<char, _, _>(text::digits(10))
        .collect::<String>()
        .map(|s: String| Token::Float(s));

    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(|s: String| Token::String(s));

    let operator = choice((
        just("+"),
        just("-"),
        just("*"),
        just("/"),
        just("%"),

        just("!"),
        just("=="),
        just("!="),
        just("<"),
        just(">"),
        just("<="),
        just(">="),
    )).map(|c| Token::Operator(c.to_string()));

    let delimiter = choice((
        just('('),
        just(')'),
        just('{'),
        just('}'),
    )).map(|c| Token::Delimiter(c));

    let symbol = choice((
        just(';').to(Token::Semicolon),
        just('=').to(Token::Assign),
        just(':').to(Token::Colon),
        just(',').to(Token::Comma),
    ));

    let keyword = text::ident().map(|s: String| match s.as_str() {
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),

        "let" => Token::Let,
        "fun" => Token::Fun,
        _ => Token::Ident(s),
    });

    let token = int
        .or(float)
        .or(string)
        .or(operator)
        .or(delimiter)
        .or(symbol)
        .or(keyword)
        .recover_with(skip_then_retry_until([]));

    let comment = just("/*").then(take_until(just("*/")))
        .padded()
        .ignored();

    token
        .padded_by(comment.repeated())
        .map_with_span(|token, span| (token, span))
        .padded()
        .repeated()
}

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64), Float(f64),
    Boolean(bool), String(String),
    Ident(String),

    Unary { op: String, expr: Box<Self> },
    Binary { op: String, left: Box<Self>, right: Box<Self> },

    Let {
        name: String,
        value: Vec<Self>,
    },
    Fun {
        name: String,
        args: Vec<String>,
        body: Vec<Self>,
    },
    Call {
        name: Box<Self>,
        args: Vec<Self>,
    },
}

fn expr_parser() -> impl Parser<Token, Expr, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, token| match token {
        Token::Ident(s) => Ok(s.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    }).labelled("identifier");

    let expr = recursive(|expr| {
        let literal = filter_map(|span, token| match token {
            Token::Int(i) => Ok(Expr::Int(i)),
            Token::Float(f) => Ok(Expr::Float(f.parse().unwrap())),
            Token::Boolean(b) => Ok(Expr::Boolean(b)),
            Token::String(s) => Ok(Expr::String(s)),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
        }).labelled("literal");

        let args = expr.clone()
            .chain(just(Token::Comma)
                .ignore_then(expr.clone()).repeated())
            .then_ignore(just(Token::Comma).or_not())
            .or_not()
            .map(|item| item.unwrap_or_else(Vec::new));

        let atom = literal
            .or(ident.map(Expr::Ident))
            .or(
                expr.clone()
                .delimited_by(just(Token::Delimiter('(')), just(Token::Delimiter(')'))))
            .labelled("atom");

        let call = atom
            .then(
                args
                    .delimited_by(
                        just(Token::Delimiter('(')),
                        just(Token::Delimiter(')')))
                    .repeated()
            )
            .foldl(|f, args| {
                Expr::Call {
                    name: Box::new(f),
                    args,
                }
            });
        
        let unary =  choice((
                just(Token::Operator("-".to_string())).to("-"),
                just(Token::Operator("!".to_string())).to("!")))
            .repeated()
            .then(call)
            .foldr(|op, rhs| Expr::Unary { op: op.to_string(), expr: Box::new(rhs) }).labelled("unary");
        
        let factor = unary.clone()
            .then(
                choice((
                    just(Token::Operator("*".to_string())).to("*"),
                    just(Token::Operator("/".to_string())).to("/")))
                .then(unary)
                .repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op: op.to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs)
            }).labelled("factor");
        
        let term = factor.clone()
            .then(
                choice((
                    just(Token::Operator("+".to_string())).to("+"),
                    just(Token::Operator("-".to_string())).to("-")))
                .then(factor)
                .repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op: op.to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs)
            }).labelled("term");

        let compare = term.clone()
            .then(
                choice((
                    just(Token::Operator("==".to_string())).to("=="),
                    just(Token::Operator("!=".to_string())).to("!="),
                    just(Token::Operator("<".to_string())).to("<"),
                    just(Token::Operator(">".to_string())).to(">"),
                    just(Token::Operator("<=".to_string())).to("<="),
                    just(Token::Operator(">=".to_string())).to(">=")))
                .then(term)
                .repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op: op.to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs)
            }).labelled("compare");
        
        compare
    }).labelled("expression");

    let declare = recursive(|decl| {
        let decl_block = decl.clone()
            .or(expr.clone())
            .repeated()
            .delimited_by(just(Token::Delimiter('{')), just(Token::Delimiter('}')));

        let declare_var = just(Token::Let)
            .ignore_then(ident)
            .then_ignore(just(Token::Assign))
            .then(
                decl_block.clone()
                    .or(decl.clone().repeated().at_most(1))
            )
            .then_ignore(just(Token::Semicolon))
            .map(|(name, value)| Expr::Let {
                name,
                value,
            });

        let declare_fun = just(Token::Fun)
            .ignore_then(ident)
            .then(ident.repeated())
            .then_ignore(just(Token::Assign))
            .then(
                decl_block.clone()
                    .or(decl.clone().repeated().at_most(1))
            )
            .then_ignore(just(Token::Semicolon))
            .map(|((name, args), body)| Expr::Fun {
                name,
                args,
                body,
            });

        declare_var
            .or(declare_fun)
            .or(expr)
    });

    declare
}

pub fn parser() -> impl Parser<Token, Vec<Expr>, Error = Simple<Token>> + Clone {
    expr_parser()
        .repeated()
        .then_ignore(end())
}