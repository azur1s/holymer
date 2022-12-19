#![feature(trait_alias)]
#![allow(clippy::type_complexity)]
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{error, prelude::*, Stream};

pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Delimiter {
    Paren,
    Brack,
    Brace,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Token {
    Num(i64),
    Bool(bool),
    Str(String),
    Sym(String),

    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Not,
    Pipe,
    Assign,
    Arrow,
    Backslash,
    Comma,
    Semi,
    Open(Delimiter),
    Close(Delimiter),

    Fun,
    Let,
    In,
    If,
    Then,
    Else,
    Do,
    End,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Num(n) => write!(f, "{}", n),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Str(s) => write!(f, "{}", s),
            Token::Sym(s) => write!(f, "{}", s),

            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),

            Token::Lt => write!(f, "<"),
            Token::Le => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::Ge => write!(f, ">="),
            Token::Eq => write!(f, "=="),
            Token::Ne => write!(f, "!="),

            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),

            Token::Pipe => write!(f, "|>"),

            Token::Assign => write!(f, "="),
            Token::Arrow => write!(f, "->"),
            Token::Backslash => write!(f, "\\"),
            Token::Comma => write!(f, ","),
            Token::Semi => write!(f, ";"),

            Token::Open(d) => write!(
                f,
                "{}",
                match d {
                    Delimiter::Paren => "(",
                    Delimiter::Brack => "[",
                    Delimiter::Brace => "{",
                }
            ),
            Token::Close(d) => write!(
                f,
                "{}",
                match d {
                    Delimiter::Paren => ")",
                    Delimiter::Brack => "]",
                    Delimiter::Brace => "}",
                }
            ),

            Token::Fun => write!(f, "fun"),
            Token::Let => write!(f, "let"),
            Token::In => write!(f, "in"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Do => write!(f, "do"),
            Token::End => write!(f, "end"),
        }
    }
}

pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let int = text::int(10).map(|s: String| Token::Num(s.parse().unwrap()));

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
        just("|>").to(Token::Pipe),
        just("<=").to(Token::Le),
        just('<').to(Token::Lt),
        just(">=").to(Token::Ge),
        just('>').to(Token::Gt),
        just("!=").to(Token::Ne),
        just("==").to(Token::Eq),
        just("&&").to(Token::And),
        just("||").to(Token::Or),
        just('!').to(Token::Not),
        just('=').to(Token::Assign),
        just('\\').to(Token::Backslash),
        just(',').to(Token::Comma),
        just(';').to(Token::Semi),
    ));

    let delim = choice((
        just('(').to(Token::Open(Delimiter::Paren)),
        just(')').to(Token::Close(Delimiter::Paren)),
        just('[').to(Token::Open(Delimiter::Brack)),
        just(']').to(Token::Close(Delimiter::Brack)),
        just('{').to(Token::Open(Delimiter::Brace)),
        just('}').to(Token::Close(Delimiter::Brace)),
    ));

    let keyword = text::ident().map(|s: String| match s.as_str() {
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),

        "fun" => Token::Fun,
        "let" => Token::Let,
        "in" => Token::In,
        "if" => Token::If,
        "then" => Token::Then,
        "else" => Token::Else,
        "do" => Token::Do,
        "end" => Token::End,
        _ => Token::Sym(s),
    });

    let token = int
        .or(string)
        .or(symbol)
        .or(delim)
        .or(keyword)
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Num(i64),
    Bool(bool),
    Str(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Pipe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Error,

    Literal(Literal),
    Sym(String),
    Vec(Vec<Spanned<Self>>),
    Unary(Spanned<UnaryOp>, Box<Spanned<Self>>),
    Binary(Spanned<BinaryOp>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Lambda(Vec<String>, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    Let(Vec<(String, Spanned<Self>)>, Option<Box<Spanned<Self>>>),
    If(
        Box<Spanned<Self>>,
        Box<Spanned<Self>>,
        Option<Box<Spanned<Self>>>,
    ),
    Do(Vec<Spanned<Expr>>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Fun(String, Vec<String>, Spanned<Expr>),
}

pub trait P<T> = chumsky::Parser<Token, T, Error = Simple<Token>> + Clone;

pub fn literal_parser() -> impl P<Literal> {
    filter_map(|span, token| match token {
        Token::Num(i) => Ok(Literal::Num(i)),
        Token::Bool(b) => Ok(Literal::Bool(b)),
        Token::Str(s) => Ok(Literal::Str(s)),
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
    delim: Delimiter,
    f: impl Fn(Span) -> T + Clone + 'a,
) -> impl P<T> + 'a {
    parser
        .delimited_by(just(Token::Open(delim)), just(Token::Close(delim)))
        .recover_with(nested_delimiters(
            Token::Open(delim),
            Token::Close(delim),
            [
                (
                    Token::Open(Delimiter::Paren),
                    Token::Close(Delimiter::Paren),
                ),
                (
                    Token::Open(Delimiter::Brack),
                    Token::Close(Delimiter::Brack),
                ),
                (
                    Token::Open(Delimiter::Brace),
                    Token::Close(Delimiter::Brace),
                ),
            ],
            f,
        ))
        .boxed()
}

pub fn expr_parser() -> impl P<Spanned<Expr>> {
    recursive(|expr| {
        let lit = literal_parser().map(Expr::Literal);
        let ident = symbol_parser().map(Expr::Sym);

        let vec = nested_parser(
            expr.clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .map(Some),
            Delimiter::Brack,
            |_| None,
        )
        .map(|elems| match elems {
            Some(elems) => Expr::Vec(elems),
            None => Expr::Vec(Vec::new()),
        })
        .labelled("vector");

        let paren_expr = just(Token::Open(Delimiter::Paren))
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Close(Delimiter::Paren)))
            .map(|e| e.0)
            .labelled("parenthesized expression");

        let lam = just(Token::Backslash)
            .ignore_then(symbol_parser().repeated())
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .map(|(args, body)| Expr::Lambda(args, Box::new(body)))
            .labelled("lambda");

        let let_binds = symbol_parser()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|(sym, expr)| (sym, expr))
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .labelled("let bindings");

        let let_in = just(Token::Let)
            .ignore_then(let_binds.clone())
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map(|(binds, body)| Expr::Let(binds, Some(Box::new(body))))
            .boxed()
            .labelled("let..in");

        let let_def = just(Token::Let)
            .ignore_then(let_binds)
            .map(|binds| Expr::Let(binds, None))
            .labelled("let");

        let if_ = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(expr.clone())
            .then(just(Token::Else).ignore_then(expr.clone()).or_not())
            .map(|((cond, then), else_)| {
                Expr::If(Box::new(cond), Box::new(then), else_.map(Box::new))
            });

        let block = just(Token::Do)
            .ignore_then(expr.clone().repeated())
            .then_ignore(just(Token::End))
            .map(Expr::Do)
            .labelled("do block");

        let atom = lit
            .or(ident)
            .or(vec)
            .or(paren_expr)
            .or(lam)
            .or(let_in)
            .or(let_def)
            .or(if_)
            .or(block)
            .map_with_span(|e, s| (e, s))
            .boxed()
            .labelled("atom");

        let call = atom
            .then(
                nested_parser(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .map(Some),
                    Delimiter::Paren,
                    |_| None,
                )
                .or_not(),
            )
            .map_with_span(|(f, args), s| match args {
                Some(Some(args)) => (Expr::Call(Box::new(f), args), s),
                Some(None) => (Expr::Error, s),
                None => f,
            });

        let unary = choice((
            just(Token::Sub).to(UnaryOp::Neg),
            just(Token::Not).to(UnaryOp::Not),
        ))
        .map_with_span(|op, s| (op, s))
        .repeated()
        .then(call)
        .foldr(|op, expr| {
            let s = op.1.start()..expr.1.end();
            (Expr::Unary(op, Box::new(expr)), s)
        })
        .boxed();

        let product = unary
            .clone()
            .then(
                choice((
                    just(Token::Mul).to(BinaryOp::Mul),
                    just(Token::Div).to(BinaryOp::Div),
                ))
                .map_with_span(|op, s| (op, s))
                .then(unary)
                .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.1.start()..rhs.1.end();
                (Expr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
            })
            .boxed();

        let sum = product
            .clone()
            .then(
                choice((
                    just(Token::Add).to(BinaryOp::Add),
                    just(Token::Sub).to(BinaryOp::Sub),
                ))
                .map_with_span(|op, s| (op, s))
                .then(product)
                .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.1.start()..rhs.1.end();
                (Expr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
            })
            .boxed();

        let comparison = sum
            .clone()
            .then(
                choice((
                    just(Token::Eq).to(BinaryOp::Eq),
                    just(Token::Ne).to(BinaryOp::Ne),
                    just(Token::Lt).to(BinaryOp::Lt),
                    just(Token::Le).to(BinaryOp::Le),
                    just(Token::Gt).to(BinaryOp::Gt),
                    just(Token::Ge).to(BinaryOp::Ge),
                ))
                .map_with_span(|op, s| (op, s))
                .then(sum)
                .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.1.start()..rhs.1.end();
                (Expr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
            })
            .boxed();

        let logical = comparison
            .clone()
            .then(
                choice((
                    just(Token::And).to(BinaryOp::And),
                    just(Token::Or).to(BinaryOp::Or),
                ))
                .map_with_span(|op, s| (op, s))
                .then(comparison)
                .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.1.start()..rhs.1.end();
                (Expr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
            })
            .boxed();

        logical
            .clone()
            .then(
                just(Token::Pipe)
                    .to(BinaryOp::Pipe)
                    .map_with_span(|op, s| (op, s))
                    .then(logical)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let s = lhs.1.start()..rhs.1.end();
                (Expr::Binary(op, Box::new(lhs), Box::new(rhs)), s)
            })
            .boxed()
    })
}

pub fn stmt_parser() -> impl P<Spanned<Stmt>> {
    let fun = just(Token::Fun)
        .ignore_then(symbol_parser())
        .then(symbol_parser().repeated())
        .then_ignore(just(Token::Assign))
        .then(expr_parser())
        .map(|((name, args), body)| Stmt::Fun(name, args, body));

    fun.map_with_span(|e, s| (e, s))
}

pub fn stmts_parser() -> impl P<Vec<Spanned<Stmt>>> {
    stmt_parser().repeated()
}

pub fn parse(
    tokens: Vec<Spanned<Token>>,
    len: usize,
) -> (Option<Vec<Spanned<Stmt>>>, Vec<Simple<Token>>) {
    let (ast, parse_error) = stmts_parser()
        .then_ignore(end())
        .parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

    (ast, parse_error)
}

pub fn parse_expr(
    tokens: Vec<Spanned<Token>>,
    len: usize,
) -> (Option<Spanned<Expr>>, Vec<Simple<Token>>) {
    let (ast, parse_error) = expr_parser()
        .then_ignore(end())
        .parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

    (ast, parse_error)
}

pub fn report(e: Simple<String>, src: &str) {
    let report = Report::build(ReportKind::Error, (), e.span().start());

    let report = match e.reason() {
        error::SimpleReason::Unclosed { span, delimiter } => report
            .with_message("Unclosed delimiter")
            .with_label(
                Label::new(span.clone())
                    .with_message(format!("Unclosed {}", delimiter.fg(Color::Yellow)))
                    .with_color(Color::Yellow),
            )
            .with_label(
                Label::new(e.span())
                    .with_message(format!(
                        "Delimiter must be closed before {}",
                        e.found()
                            .unwrap_or(&"end of file".to_string())
                            .fg(Color::Red)
                    ))
                    .with_color(Color::Red),
            ),

        error::SimpleReason::Unexpected => report
            .with_message(format!(
                "Unexpected {}, expected {}",
                if e.found().is_some() {
                    "token in input"
                } else {
                    "end of input"
                },
                if e.expected().len() == 0 {
                    "something else".to_string()
                } else {
                    e.expected()
                        .map(|expected| match expected {
                            Some(expected) => expected.to_string(),
                            None => "end of input".to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ))
            .with_label(
                Label::new(e.span())
                    .with_message(format!(
                        "Unexpected token {}",
                        e.found()
                            .unwrap_or(&"end of file".to_string())
                            .fg(Color::Red)
                    ))
                    .with_color(Color::Red),
            ),
        chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
            Label::new(e.span())
                .with_message(format!("{}", msg.fg(Color::Red)))
                .with_color(Color::Red),
        ),
    };

    report.finish().eprint(Source::from(&src)).unwrap();
}
