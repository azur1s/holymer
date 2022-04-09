use chumsky::{prelude::*, Stream};
use lexer::Token;

pub mod types;
use types::{Expr, Spanned, Typehint};

fn typehint_parser() -> impl Parser<Token, Spanned<Typehint>, Error = Simple<Token>> + Clone {
    recursive(|ty| {
        let single = filter_map(|span, token| match token {
            Token::Identifier(s) => Ok((Typehint::Single(s), span)),
            _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
        });

        let tuple = single
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(
                just(Token::OpenParen),
                just(Token::CloseParen),
            )
            .map_with_span(|args, span| {
                (Typehint::Tuple(args), span)
            });

        let vector = single
            .delimited_by(
                just(Token::OpenBracket),
                just(Token::CloseBracket),
            )
            .map_with_span(|arg, span| {
                (Typehint::Vector(Box::new(arg)), span)
            });

        let function = ty.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(
                just(Token::Pipe),
                just(Token::Pipe),
            )
            .then_ignore(just(Token::Arrow))
            .then(ty)
            .map_with_span(|(args, ret), span| {
                (Typehint::Function(args, Box::new(ret)), span)
            });

        single
            .or(tuple)
            .or(vector)
            .or(function)
            .labelled("type hint")
    })
}

fn expr_parser() -> impl Parser<Token, Vec<Spanned<Expr>>, Error = Simple<Token>> + Clone {
    let identifier = filter_map(|span, token| match token {
        Token::Identifier(s) => Ok((s, span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    }).labelled("identifier");

    let literal = filter_map(|span: std::ops::Range<usize>, token| match token {
        Token::Int(i)     => Ok((Expr::Int(i), span)),
        Token::Boolean(b) => Ok((Expr::Boolean(b), span)),
        Token::String(s)  => Ok((Expr::String(s), span)),
        Token::Hole       => Ok((Expr::Hole(span.start, span.end), span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    }).labelled("literal");

    let expr = recursive(|expr| {
        let args = expr.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing();

        let vector = expr.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(
                just(Token::OpenBracket),
                just(Token::CloseBracket),
            )
            .map_with_span(|args, span| {
                (
                    Expr::Vector(args),
                    span,
                )
            });

        let tuple = expr.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .delimited_by(
                just(Token::OpenParen),
                just(Token::CloseParen),
            )
            .map_with_span(|args, span| {
                (
                    Expr::Tuple(args),
                    span,
                )
            });

        let atom = literal
            .or(identifier.map(|(s, span)| (Expr::Identifier(s), span)))
            .or(vector)
            .or(tuple)
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
                    just(Token::Divide),
                    just(Token::Modulus)))
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

        let pipeline = compare.clone()
            .then(
                just(Token::Pipeline)
                .ignore_then(compare)
                .repeated())
            .foldl(|lhs, rhs| {
                (
                    Expr::Pipeline {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs.clone()),
                    },
                    rhs.1,
                )
            });

        let let_ = just(Token::KwPub).or_not()
            .then_ignore(just(Token::KwLet))
            .then(just(Token::KwMut).or_not())
            .then(identifier)
            .then_ignore(just(Token::Colon))
            .then(typehint_parser())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|((((public, mutable), name), type_hint), value)| {
                (
                    Expr::Let {
                        public: public.is_some(),
                        name: name.clone(),
                        type_hint,
                        value: Box::new(value.clone()),
                        mutable: mutable.is_some(),
                    },
                    name.1.start..value.1.end,
                )
            });

        let fun = just(Token::KwPub).or_not()
            .then_ignore(just(Token::KwFun))
            .then(identifier)
            .then(
                identifier
                    .then_ignore(just(Token::Colon))
                    .then(typehint_parser())
                    .delimited_by(
                        just(Token::OpenParen),
                        just(Token::CloseParen),
                    )
                    .repeated()
            )
            .then_ignore(just(Token::Colon))
            .then(typehint_parser())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|((((public, name), args), type_hint), body)| {
                (
                    Expr::Fun {
                        public: public.is_some(),
                        name: name.clone(),
                        type_hint,
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

        let do_block = expr.clone()
            .repeated()
            .delimited_by(
                just(Token::KwDo),
                just(Token::KwEnd),
            )
            .map_with_span(|body, span| {
                (
                    Expr::Do {
                        body: (body, span.clone()),
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

        let match_ = just(Token::KwMatch)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::KwWith))
            .then(
                just(Token::Pipe)
                    .ignore_then(expr.clone())
                    .then_ignore(just(Token::Arrow))
                    .then(expr.clone())
                    .repeated()
            )
            .then(
                just(Token::Pipe)
                    .ignore_then(just(Token::KwElse))
                    .ignore_then(expr.clone())
            )
            .then_ignore(just(Token::KwEnd))
            .map(|((expr, cases), default)| {
                (
                    Expr::Case {
                        expr: Box::new(expr.clone()),
                        cases: (
                            cases.clone(),
                            cases.first().unwrap().1.start()..cases.last().unwrap().1.end()
                        ),
                        default: Box::new(default.clone()),
                    },
                    expr.1.start()..default.1.end(),
                )
            });

        let_
            .or(fun)
            .or(return_)
            .or(do_block)
            .or(if_block)
            .or(match_)
            .or(pipeline)
    }).labelled("expression");

    expr
        .repeated()
        .then_ignore(end())
}

#[allow(clippy::type_complexity)] // We are going to use this once anyway, why we need to make a type?
pub fn parse(tokens: Vec<(Token, std::ops::Range<usize>)>, len: usize) -> (Option<Vec<(Expr, std::ops::Range<usize>)>>, Vec<Simple<Token>>) {
    let (ast, parse_error) = expr_parser().parse_recovery(Stream::from_iter(
        len..len + 1,
        tokens.into_iter(),
    ));

    (ast, parse_error)
}