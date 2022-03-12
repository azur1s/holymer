use chumsky::{prelude::*, Stream};
use lexer::Token;

pub type Spanned<T> = (T, std::ops::Range<usize>);

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64), Float(f64), Boolean(bool),
    String(String), Identifier(String),

    Unary { op: String, rhs: Box<Spanned<Self>> },
    Binary { lhs: Box<Spanned<Self>>, op: String, rhs: Box<Spanned<Self>> },
    Call { name: Box<Spanned<Self>>, args: Spanned<Vec<Spanned<Self>>> },
    Intrinsic { name: Box<Spanned<Self>>, args: Spanned<Vec<Spanned<Self>>> },

    Let {
        name: String,
        type_hint: String,
        value: Box<Spanned<Self>>,
    },
    Fun {
        name: String,
        type_hint: String,
        args: Spanned<Vec<(Spanned<String>, Spanned<String>)>>,
        body: Box<Spanned<Self>>
    },
    Return { expr: Box<Spanned<Self>> },

    If {
        cond: Box<Spanned<Self>>,
        body: Box<Spanned<Self>>,
        else_body: Box<Spanned<Self>>
    },
    Do {
        body: Vec<Spanned<Self>>
    },
}

fn expr_parser() -> impl Parser<Token, Vec<Spanned<Expr>>, Error = Simple<Token>> + Clone {
    let identifier = filter_map(|span, token| match token {
        Token::Identifier(s) => Ok((s, span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    }).labelled("identifier");

    let literal = filter_map(|span, token| match token {
        Token::Int(i)     => Ok((Expr::Int(i), span)),
        Token::Boolean(b) => Ok((Expr::Boolean(b), span)),
        Token::String(s)  => Ok((Expr::String(s), span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    }).labelled("literal");

    let expr = recursive(|expr| {
        let args = expr.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing();

        let atom = literal
            .or(identifier.map(|(s, span)| (Expr::Identifier(s), span)))
            // .or(
            //     expr.clone()
            //     .delimited_by(just(Token::OpenParen), just(Token::CloseParen)))
            .labelled("atom");

        let call = atom.clone()
            .then(
                args.clone()
                    .delimited_by(
                        just(Token::OpenParen),
                        just(Token::CloseParen),
                    )
                    .repeated()
            )
            .foldl(|name, args| {
                (
                    Expr::Call {
                        name: Box::new(name.clone()),
                        args: (args, name.1.clone()),
                    },
                    name.1,
                )
            });

        let intrinsic = just(Token::At) 
            .ignore_then(atom)
            .then(
                args.clone()
                    .delimited_by(
                        just(Token::OpenParen),
                        just(Token::CloseParen),
                    )
                    .repeated()
            )
            .foldl(|name, args| {
                (
                    Expr::Intrinsic {
                        name: Box::new(name.clone()),
                        args: (args, name.1.clone()),
                    },
                    name.1,
                )
            });

        let unary =  choice((
                just(Token::Plus),
                just(Token::Minus)))
            .repeated()
            .then(call.or(intrinsic))
            .foldr(|op, rhs| {
                (
                    Expr::Unary {
                        op: op.to_string(),
                        rhs: Box::new(rhs.clone()),
                    },
                    rhs.1,
                )
            });

        let factor = unary.clone()
            .then(
                choice((
                    just(Token::Multiply),
                    just(Token::Divide)))
                .then(unary)
                .repeated())
            .foldl(|lhs, (op, rhs)| {
                (
                    Expr::Binary {
                        lhs: Box::new(lhs),
                        op: op.to_string(),
                        rhs: Box::new(rhs.clone()),
                    },
                    rhs.1,
                )
            });

        let term = factor.clone()
            .then(
                choice((
                    just(Token::Plus),
                    just(Token::Minus)))
                .then(factor)
                .repeated())
            .foldl(|lhs, (op, rhs)| {
                (
                    Expr::Binary {
                        lhs: Box::new(lhs),
                        op: op.to_string(),
                        rhs: Box::new(rhs.clone()),
                    },
                    rhs.1,
                )
            });

        let compare = term.clone()
            .then(
                choice((
                    just(Token::Less),
                    just(Token::Greater),
                    just(Token::Equal),
                    just(Token::NotEqual)))
                .then(term)
                .repeated())
            .foldl(|lhs, (op, rhs)| {
                (
                    Expr::Binary {
                        lhs: Box::new(lhs),
                        op: op.to_string(),
                        rhs: Box::new(rhs.clone()),
                    },
                    rhs.1,
                )
            });

        let let_ = just(Token::KwLet)
            .ignore_then(identifier)
            .then_ignore(just(Token::Colon))
            .then(identifier)
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|((name, type_hint), value)| {
                (
                    Expr::Let {
                        name: name.0.clone(),
                        type_hint: type_hint.0,
                        value: Box::new(value.clone()),
                    },
                    name.1.start..value.1.end,
                )
            });

        let fun = just(Token::KwFun)
            .ignore_then(identifier)
            .then(
                identifier
                    .then_ignore(just(Token::Colon))
                    .then(identifier)
                    .delimited_by(
                        just(Token::OpenParen),
                        just(Token::CloseParen),
                    )
                    .repeated()
            )
            .then_ignore(just(Token::Colon))
            .then(identifier)
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|(((name, args), type_hint), body)| {
                (
                    Expr::Fun {
                        name: name.0.clone(),
                        type_hint: type_hint.0,
                        args: (args, name.1.clone()),
                        body: Box::new(body.clone()),
                    },
                    name.1.start..body.1.end,
                )
            });

        let return_ = just(Token::KwReturn)
            .ignore_then(expr.clone())
            .map(|(expr, span)| {
                (
                    Expr::Return {
                        expr: Box::new((expr, span.clone())),
                    },
                    span.start..span.end,
                )
            });

        let do_block = just(Token::KwDo)
            .ignore_then(
                expr.clone()
                    .then_ignore(just(Token::SemiColon))
                    .repeated()
            )
            .then_ignore(just(Token::KwEnd))
            .map_with_span(|body, span| {
                (
                    Expr::Do {
                        body: body.clone(),
                    },
                    span,
                )
            });

        let if_block = just(Token::KwIf)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::KwThen))
            .then(
                do_block.clone()
                    .or(expr.clone())
            )
            .then_ignore(just(Token::KwElse))
            .then(
                do_block.clone()
                    .or(expr.clone())
            )
            .then_ignore(just(Token::KwEnd))
            .map(|((cond, then), else_)| {
                (
                    Expr::If {
                        cond: Box::new(cond.clone()),
                        body: Box::new(then),
                        else_body: Box::new(else_.clone()),
                    },
                    cond.1.start..else_.1.end,
                )
            });

        let_
            .or(fun)
            .or(return_)
            .or(do_block)
            .or(if_block)
            .or(compare)
    }).labelled("expression");

    expr
        .then_ignore(just(Token::SemiColon))
        .repeated()
        .then_ignore(end())
}

pub fn parse(tokens: Vec<(Token, std::ops::Range<usize>)>, len: usize) -> (Option<Vec<(Expr, std::ops::Range<usize>)>>, Vec<Simple<Token>>) {
    let (ast, parse_error) = expr_parser().parse_recovery(Stream::from_iter(
        len..len + 1,
        tokens.into_iter(),
    ));

    return (ast, parse_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let (_, err) = parse(vec![
            (Token::KwLet, 0..3),
            (Token::Identifier("x".to_string()), 4..5),
            (Token::Colon, 5..6),
            (Token::Identifier("Int".to_string()), 7..10),
            (Token::Assign, 11..12),
            (Token::Int(1), 13..14),
            (Token::SemiColon, 14..15),
        ], 15);

        assert_eq!(err, vec![]);
    }
}