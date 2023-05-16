use chumsky::prelude::*;

use super::{ expr::*, ty::Type };

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Rich<'src, char, Span>>> {
    // let num = text::int(10)
    //     .then(just('.').then(text::digits(10)).or_not())
    //     .slice()
    //     .from_str()
    //     .unwrapped()
    //     .map(Token::Int);
    let int = text::int(10)
        .slice()
        .from_str()
        .unwrapped()
        .map(Token::Int);

    let strn = just('"')
        .ignore_then(none_of('"').repeated())
        .then_ignore(just('"'))
        .map_slice(Token::Str);

    fn id_filter<C>(c: &C) -> bool where C: text::Char {
        c.to_char().is_ascii_alphabetic()
        || "_'".contains(c.to_char())
    }
    let id = any()
        .filter(id_filter)
        .then(any()
            .filter(id_filter)
            .repeated())
        .slice();

    let word = id.map(|s: &str| match s {
        "true"   => Token::Bool(true),
        "false"  => Token::Bool(false),
        "let"    => Token::Let,
        "in"     => Token::In,
        "fun"    => Token::Func,
        "return" => Token::Return,
        "if"     => Token::If,
        "then"   => Token::Then,
        "else"   => Token::Else,
        _        => Token::Ident(s),
    });

    let sym = choice((
        just("()").to(Token::Unit),
        just("\\").to(Token::Lambda),
        just("->").to(Token::Arrow),
        just("|>").to(Token::Pipe),

        just('+').to(Token::Add),
        just('-').to(Token::Sub),
        just('*').to(Token::Mul),
        just('/').to(Token::Div),
        just('%').to(Token::Rem),
        just("==").to(Token::Eq),
        just("!=").to(Token::Ne),
        just("<=").to(Token::Le),
        just(">=").to(Token::Ge),
        just('<').to(Token::Lt),
        just('>').to(Token::Gt),
        just("&&").to(Token::And),
        just("||").to(Token::Or),
        just('!').to(Token::Not),

        just('=').to(Token::Assign),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just(';').to(Token::Semicolon),
    ));

    let delim = choice((
        just('(').to(Token::Open(Delim::Paren)),
        just(')').to(Token::Close(Delim::Paren)),
        just('[').to(Token::Open(Delim::Brack)),
        just(']').to(Token::Close(Delim::Brack)),
        just('{').to(Token::Open(Delim::Brace)),
        just('}').to(Token::Close(Delim::Brace)),
    ));

    let token = choice((
            int,
            strn,
            word,
            sym,
            delim,
        ));

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with_span(move |tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded()
        // If we get an error, skip to the next character and try again.
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

// (a, s) -> (Box::new(a), s)
fn boxspan<T>(a: Spanned<T>) -> Spanned<Box<T>> {
    (Box::new(a.0), a.1)
}

// Lifetime 'tokens is the lifetime of the token buffer from the lexer.
type ParserInput<'tokens, 'src> =
    chumsky::input::SpannedInput<
        Token<'src>,
        Span,
        &'tokens [(Token<'src>, Span)]
    >;

pub fn expr_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Spanned<Expr<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    recursive(|expr| {
        let lit = select! {
            Token::Unit    => Expr::Lit(Lit::Unit),
            Token::Bool(b) => Expr::Lit(Lit::Bool(b)),
            Token::Int(n)  => Expr::Lit(Lit::Int(n)),
            Token::Str(s)  => Expr::Lit(Lit::Str(s)),
        };

        let symbol = select! {
            Token::Ident(s) => s,
        };

        let ident = symbol
            .map(Expr::Ident);

        let paren_expr = expr.clone()
            .delimited_by(
                just(Token::Open(Delim::Paren)),
                just(Token::Close(Delim::Paren)),
            )
            .map(|e: Spanned<Expr>| e.0);

        let lambda = just(Token::Func)
            .ignore_then(
                symbol
                    .then(type_parser().or_not())
                    .separated_by(just(Token::Comma))
                    .collect::<Vec<_>>()
                    .delimited_by(
                        just(Token::Open(Delim::Paren)),
                        just(Token::Close(Delim::Paren)),
                    )
            )
            .then(type_parser().or_not())
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .map(|((args, ret), body)| Expr::Lambda(args, ret, boxspan(body)));

        // ident (: type)?
        let bind = symbol
            .then(
                just(Token::Colon)
                    .ignore_then(type_parser())
                    .or_not()
            )
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|((name, ty), expr)| (name, ty, boxspan(expr)));

        let let_or_define = just(Token::Let)
            .ignore_then(bind)
            .then(
                just(Token::In)
                    .ignore_then(expr.clone())
                    .or_not()
            )
            .map(|((name, ty, expr), body)| match body {
                Some(body) => Expr::Let { name, ty, value: expr, body: boxspan(body) },
                None => Expr::Define { name, ty, value: expr },
            });

        let if_ = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(expr.clone())
            .then_ignore(just(Token::Else))
            .then(expr.clone())
            .map(|((cond, t), f)| Expr::If {
                cond: boxspan(cond),
                t: boxspan(t),
                f: boxspan(f)
            });

        let block = expr.clone()
            .map(boxspan)
            .then_ignore(just(Token::Semicolon))
            .repeated()
            .collect::<Vec<_>>()
            .then(expr.clone()
                .map(boxspan)
                .or_not())
            .delimited_by(
                just(Token::Open(Delim::Brace)),
                just(Token::Close(Delim::Brace)),
            )
            .map(|(mut exprs, end)| {
                let void = end.is_none();
                if let Some(end) = end {
                    exprs.push(end);
                }
                Expr::Block {
                    exprs,
                    void,
                }
            });

        let atom = lit
            .or(ident)
            .or(paren_expr)
            .or(lambda)
            .or(let_or_define)
            .or(if_)
            .or(block)
            .map_with_span(|e, s| (e, s))
            .boxed()
            .labelled("(atomic) expression");

        let call = atom
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(
                        just(Token::Open(Delim::Paren)),
                        just(Token::Close(Delim::Paren)),
                    )
                    .or_not()
            )
            .map_with_span(|(f, args), s| match args {
                Some(args) => (Expr::Call(boxspan(f), args), s),
                None => (f.0, f.1),
            });

        let op = choice((
            just(Token::Sub).to(UnaryOp::Neg),
            just(Token::Not).to(UnaryOp::Not),
        ));
        let unary = op
            .map_with_span(|op, s| (op, s))
            .repeated()
            .foldr(
                call,
                |op, expr| {
                let span = op.1.start..expr.1.end;
                (Expr::Unary(op.0, boxspan(expr)), span.into())
            });

        let op = choice((
            just(Token::Mul).to(BinaryOp::Mul),
            just(Token::Div).to(BinaryOp::Div),
        ));
        let product = unary.clone()
            .foldl(
                op.then(unary).repeated(),
                |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(op, boxspan(a), boxspan(b)), span.into())
                }
            );

        let op = choice((
            just(Token::Add).to(BinaryOp::Add),
            just(Token::Sub).to(BinaryOp::Sub),
        ));
        let sum = product.clone()
            .foldl(
                op.then(product).repeated(),
                |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(op, boxspan(a), boxspan(b)), span.into())
                }
            );

        let op = choice((
            just(Token::Eq).to(BinaryOp::Eq),
            just(Token::Ne).to(BinaryOp::Ne),
            just(Token::Lt).to(BinaryOp::Lt),
            just(Token::Le).to(BinaryOp::Le),
            just(Token::Gt).to(BinaryOp::Gt),
            just(Token::Ge).to(BinaryOp::Ge),
        ));
        let comparison = sum.clone()
            .foldl(
                op.then(sum).repeated(),
                |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(op, boxspan(a), boxspan(b)), span.into())
                }
            );

        let op = choice((
            just(Token::And).to(BinaryOp::And),
            just(Token::Or).to(BinaryOp::Or),
        ));
        let logical = comparison.clone()
            .foldl(
                op.then(comparison).repeated(),
                |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(op, boxspan(a), boxspan(b)), span.into())
                }
            );

        let pipe = logical.clone()
            .foldl(
                just(Token::Pipe).to(BinaryOp::Pipe)
                .then(logical).repeated(),
                |a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(op, boxspan(a), boxspan(b)), span.into())
                }
            );

        pipe
            .labelled("expression")
    })
}

pub fn type_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Type,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    recursive(|ty| {
        let lit_ty = select! {
            Token::Ident("Bool") => Type::Bool,
            Token::Ident("Int")  => Type::Int,
            Token::Ident("Str")  => Type::Str,
            // TODO: Support type variables in both the parser and the type checker.
            Token::Ident(_)      => Type::Var(69),
            Token::Unit          => Type::Unit,
        }.validate(|tys, span, emitter| {
            if let Type::Var(_) = tys {
                emitter.emit(Rich::custom(span,
                    "Type variables are not yet supported.".to_string()
                ));
            }
            tys
        });

        let tys_paren = ty.clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just(Token::Open(Delim::Paren)),
                just(Token::Close(Delim::Paren)),
            );

        let func = tys_paren.clone()
            .then_ignore(just(Token::Arrow))
            .then(ty.clone())
            .map(|(ta, tr)| Type::Func(ta, Box::new(tr)));

        let tuple = tys_paren
            .validate(|tys, span, emitter| {
                if tys.is_empty() {
                    emitter.emit(Rich::custom(span,
                        "Tuple must have at least one element. Use `()` for the unit type."
                        .to_string()
                    ));
                }
                tys
            })
            .map(Type::Tuple);

        let array = ty.clone()
            .delimited_by(
                just(Token::Open(Delim::Brack)),
                just(Token::Close(Delim::Brack)),
            )
            .map(|t| Type::Array(Box::new(t)));

        lit_ty
            .or(array)
            .or(func)
            .or(tuple)
            .boxed()
            .labelled("type")
    })
}

pub fn exprs_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Vec<Spanned<Expr<'src>>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    expr_parser()
        .separated_by(just(Token::Semicolon))
        .allow_trailing()
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_parser() {
        let input = "(() -> () -> () -> (num)) -> bool";
        let (ts, errs) = lexer().parse(input).into_output_errors();

        assert!(ts.is_some());
        assert!(errs.is_empty());

        if let Some(ts) = ts {
            let (ast, parse_errs) = type_parser()
                .map_with_span(|ty, span| (ty, span))
                .parse(ts.as_slice().spanned((input.len()..input.len()).into()))
                .into_output_errors();

            println!("{:?}", ast);
            println!("{:?}", parse_errs);
        }
    }

    #[test]
    fn test_expr_parser_atom() {
        let input = "
            let id : (A) -> A = (\\x -> x) in {
                if false
                    then id(3.14)
                    else id(true);
            }
        ";
        let (ast, errs) = lexer().parse(input).into_output_errors();

        assert!(ast.is_some());
        assert!(errs.is_empty());

        if let Some(ast) = ast {
            let (ast, parse_errs) = expr_parser()
                .map_with_span(|ty, span| (ty, span))
                .parse(ast.as_slice().spanned((input.len()..input.len()).into()))
                .into_output_errors();

            println!("{:?}", ast);
            println!("{:?}", parse_errs);
        }
    }
}