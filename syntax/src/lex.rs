use chumsky::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Delimiter { Paren, Bracket, Brace }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    // Keywords
    KwFun, KwSet,
    KwDo, KwEnd,
    KwIf,
    KwCase, KwOf,
    KwReturn,

    // Literals
    Int(i64), Boolean(bool),
    String(String), Identifier(String),

    // Operators
    Plus, Minus, Multiply, Divide, Modulus,
    Not, Equal, NotEqual, Less, Greater,
    Arrow, And, Or,

    // Symbols & Delimiters
    Assign, Dot, Comma, Colon, Semicolon, At, Hash,
    Open(Delimiter), Close(Delimiter),
}

pub type Span = std::ops::Range<usize>;
pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let int = text::int(10)
        .map(|s: String| Token::Int(s.parse().unwrap()));

    let string = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Token::String);

    let symbol = choice((
        just("->").to(Token::Arrow),

        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Multiply),
        just('/').to(Token::Divide),
        just('%').to(Token::Modulus),

        just('&').to(Token::And),
        just('|').to(Token::Or),

        just("!=").to(Token::NotEqual),
        just('!').or(just('¬')).to(Token::Not),
        just("==").to(Token::Equal),

        just('<').to(Token::Less),
        just('>').to(Token::Greater),

        just('=').to(Token::Assign),
        just('.').to(Token::Dot),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just(';').to(Token::Semicolon),
        just('@').to(Token::At),
        just('#').to(Token::Hash),
    ));

    let delim = choice((
        just('(').to(Token::Open(Delimiter::Paren)),
        just(')').to(Token::Close(Delimiter::Paren)),
        just('[').to(Token::Open(Delimiter::Bracket)),
        just(']').to(Token::Close(Delimiter::Bracket)),
        just('{').to(Token::Open(Delimiter::Brace)),
        just('}').to(Token::Close(Delimiter::Brace)),
    ));

    let keyword = text::ident().map(|s: String| match s.as_str() {
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),

        "fun" => Token::KwFun,
        "set" => Token::KwSet,
        "do" => Token::KwDo,
        "end" => Token::KwEnd,
        "if" => Token::KwIf,
        "case" => Token::KwCase,
        "of" => Token::KwOf,
        "return" => Token::KwReturn,
        _ => Token::Identifier(s),
    });

    let token = int
        .or(string)
        .or(symbol)
        .or(delim)
        .or(keyword)
        .recover_with(skip_then_retry_until([]));

    // let comment = just("--").then(take_until(just('\n'))).padded();
    let comment = just('-')
        .then_ignore(just('{')
            .ignore_then(none_of('}').ignored().repeated())
            .then_ignore(just("}-"))
            .or(just('-').ignore_then(none_of('\n').ignored().repeated()))
        )
        .padded()
        .ignored()
        .repeated();

    token
        .padded_by(comment)
        .map_with_span(|token, span| (token, span))
        .padded()
        .repeated()
}

#[allow(clippy::type_complexity)]
pub fn lex(src: String) -> (Option<Vec<(Token, std::ops::Range<usize>)>>, Vec<Simple<char>>) {
    let (tokens, lex_error) = lexer().parse_recovery(src.as_str());
    (tokens, lex_error)
}