pub mod expr;
pub mod parser;
pub mod ty;

#[cfg(test)]
mod tests {
    use chumsky::prelude::*;
    use super::{ expr::*, parser::* };

    #[test]
    fn simple() {
        let src = "let x = 1 + (), y = foo in x + !(y)";

        let (ts, errs) = lexer().parse(src).into_output_errors();

        assert!(errs.is_empty());
        assert_eq!(ts, Some(vec![
            (Token::Let, Span::new(0, 3)),
            (Token::Ident("x"), Span::new(4, 5)),
            (Token::Assign, Span::new(6, 7)),
            (Token::Num(1.0), Span::new(8, 9)),
            (Token::Add, Span::new(10, 11)),
            (Token::Unit, Span::new(12, 14)),
            (Token::Comma, Span::new(14, 15)),
            (Token::Ident("y"), Span::new(16, 17)),
            (Token::Assign, Span::new(18, 19)),
            (Token::Ident("foo"), Span::new(20, 23)),
            (Token::In, Span::new(24, 26)),
            (Token::Ident("x"), Span::new(27, 28)),
            (Token::Add, Span::new(29, 30)),
            (Token::Not, Span::new(31, 32)),
            (Token::Open(Delim::Paren), Span::new(32, 33)),
            (Token::Ident("y"), Span::new(33, 34)),
            (Token::Close(Delim::Paren), Span::new(34, 35)),
        ]));
    }
}