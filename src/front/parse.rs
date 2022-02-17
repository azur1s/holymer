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
    Import,
    Let, Fun,
    If, Then, Else, End,
    Do,
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

        "import" => Token::Import,
        "let" => Token::Let,
        "fun" => Token::Fun,
        "if" => Token::If,
        "then" => Token::Then,
        "else" => Token::Else,
        "end" => Token::End,
        "do" => Token::Do,
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
    Call { name: Box<Self>, args: Vec<Self> },

    Let {
        name: String,
        value: Box<Self>,
    },
    Fun {
        name: String,
        args: Vec<String>,
        body: Box<Self>,
    },

    If {
        cond: Box<Self>,
        then: Box<Self>,
        else_: Box<Self>,
    },
    Do { body: Vec<Self> },

    Import(String),
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
        let do_block = just(Token::Do)
            .ignore_then(
                expr.clone()
                    .then_ignore(just(Token::Semicolon))
                    .repeated())
            .then_ignore(just(Token::End))
            .map(|body| Expr::Do { body });

        let declare_var = just(Token::Let)
            .ignore_then(ident)
            .then_ignore(just(Token::Assign))
            .then(
                do_block.clone()
                    .or(decl.clone())
            )
            .map(|(name, value)| Expr::Let {
                name,
                value: Box::new(value),
            }).labelled("variable");

        let declare_fun = just(Token::Fun)
            .ignore_then(ident)
            .then(ident.repeated())
            .then_ignore(just(Token::Assign))
            .then(
                do_block.clone()
                    .or(decl.clone())
            )
            .map(|((name, args), body)| Expr::Fun {
                name,
                args,
                body: Box::new(body),
            }).labelled("function");

        let declare_import = just(Token::Import)
            .ignore_then(ident.clone())
            .map(Expr::Import);

        let if_cond = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(
                do_block.clone()
                    .or(decl.clone().then_ignore(just(Token::Semicolon).or_not()))
            )
            .then_ignore(just(Token::Else))
            .then(
                do_block.clone()
                .or(decl.clone().then_ignore(just(Token::Semicolon).or_not()))
            )
            .then_ignore(just(Token::End))
            .map(|((cond, then), else_)| Expr::If {
                cond: Box::new(cond),
                then: Box::new(then),
                else_: Box::new(else_),
            }).labelled("if");

        declare_var
            .or(declare_fun)
            .or(declare_import)
            .or(if_cond)
            .or(do_block)
            .or(expr)
            
    }).labelled("declare");

    declare
}

pub fn parser() -> impl Parser<Token, Vec<Expr>, Error = Simple<Token>> + Clone {
    expr_parser()
        .then_ignore(just(Token::Semicolon))
        .repeated()
        .then_ignore(end())
}