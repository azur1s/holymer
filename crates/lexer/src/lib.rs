use chumsky::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    // Keywords
    KwLet, KwFun,
    KwDo, KwEnd,
    KwIf, KwThen, KwElse,
    KwReturn,

    // Literals
    Int(i64), Boolean(bool),
    String(String), Identifier(String),

    // Operators
    Plus, Minus, Multiply, Divide, Modulus,
    Not, Equal, NotEqual, Less, Greater,
    Pipe,

    // Symbols & Delimiters
    Assign,
    Dot, Comma,
    Colon, SemiColon,
    OpenParen, CloseParen,
    At,
    Hole,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::KwLet => write!(f, "let"),
            Token::KwFun => write!(f, "fun"),
            Token::KwDo => write!(f, "do"),
            Token::KwEnd => write!(f, "end"),
            Token::KwIf => write!(f, "if"),
            Token::KwThen => write!(f, "then"),
            Token::KwElse => write!(f, "else"),
            Token::KwReturn => write!(f, "return"),

            Token::Int(i) => write!(f, "{}", i),
            Token::Boolean(b) => write!(f, "{}", b),
            Token::String(s) => write!(f, "{}", s),
            Token::Identifier(s) => write!(f, "{}", s),

            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Modulus => write!(f, "%"),
            Token::Not => write!(f, "!"),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::Pipe => write!(f, "|>"),

            Token::Assign => write!(f, "="),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::SemiColon => write!(f, ";"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::At => write!(f, "@"),
            Token::Hole => write!(f, "_"),
        }
    }
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
        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Multiply),
        just('/').to(Token::Divide),

        just('!').to(Token::Not),
        just("==").to(Token::Equal),

        just("|>").to(Token::Pipe),

        just('<').to(Token::Less),
        just('>').to(Token::Greater),

        just('=').to(Token::Assign),
        just('.').to(Token::Dot),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just(';').to(Token::SemiColon),
        just('(').to(Token::OpenParen),
        just(')').to(Token::CloseParen),
        just('@').to(Token::At),
        just('_').to(Token::Hole),
    ));

    let keyword = text::ident().map(|s: String| match s.as_str() {
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),

        "let" => Token::KwLet,
        "fun" => Token::KwFun,
        "do" => Token::KwDo,
        "end" => Token::KwEnd,
        "if" => Token::KwIf,
        "then" => Token::KwThen,
        "else" => Token::KwElse,
        "return" => Token::KwReturn,
        _ => Token::Identifier(s),
    });

    let token = int
        .or(string)
        .or(symbol)
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

pub fn lex(src: String) -> (Option<Vec<(Token, std::ops::Range<usize>)>>, Vec<Simple<char>>) {
    let (tokens, lex_error) = lexer().parse_recovery(src.as_str());
    return (tokens, lex_error);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_let_simple() {
        let (tokens, err) = lex("let x: Int = 1;".to_string());

        assert_eq!(tokens, Some(vec![
            (Token::KwLet, 0..3),
            (Token::Identifier("x".to_string()), 4..5),
            (Token::Colon, 5..6),
            (Token::Identifier("Int".to_string()), 7..10),
            (Token::Assign, 11..12),
            (Token::Int(1), 13..14),
            (Token::SemiColon, 14..15),
        ]));
        assert_eq!(err, vec![]);
    }
}