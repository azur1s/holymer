use std::fmt::{
    Display,
    Formatter,
    self,
};
use chumsky::prelude::*;
use crate::typing::ty::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum Delim { Paren, Brack, Brace }

// The tokens of the language.
// 'src is the lifetime of the source code string.
#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Unit, Bool(bool), Num(f64), Str(&'src str),
    Ident(&'src str),

    Add, Sub, Mul, Div, Rem,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or, Not,

    Assign, Comma, Colon, Semicolon,
    Open(Delim), Close(Delim),
    Lambda, Arrow,

    Let, In, Func, Return, If, Then, Else,
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Unit    => write!(f, "()"),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Num(n)  => write!(f, "{}", n),
            Token::Str(s)  => write!(f, "\"{}\"", s),
            Token::Ident(s)  => write!(f, "{}", s),

            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Rem => write!(f, "%"),
            Token::Eq  => write!(f, "=="),
            Token::Ne => write!(f, "!="),
            Token::Lt  => write!(f, "<"),
            Token::Gt  => write!(f, ">"),
            Token::Le => write!(f, "<="),
            Token::Ge => write!(f, ">="),
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

pub type Span = SimpleSpan<usize>;

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Rich<'src, char, Span>>> {
    let num = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .slice()
        .from_str()
        .unwrapped()
        .map(Token::Num);

    let strn = just('"')
        .ignore_then(none_of('"').repeated())
        .then_ignore(just('"'))
        .map_slice(Token::Str);

    let word = text::ident().map(|s: &str| match s {
        "true"   => Token::Bool(true),
        "false"  => Token::Bool(false),
        "unit"   => Token::Unit,
        "let"    => Token::Let,
        "in"     => Token::In,
        "func"   => Token::Func,
        "return" => Token::Return,
        "if"     => Token::If,
        "then"   => Token::Then,
        "else"   => Token::Else,
        _        => Token::Ident(s),
    });

    let sym = choice((
        just("\\").to(Token::Lambda),
        just("->").to(Token::Arrow),

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
            num,
            strn,
            word,
            sym,
            delim,
        ));

    token
        .map_with_span(|tok, span| (tok, span))
        .padded()
        // If we get an error, skip to the next character and try again.
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Lit<'src> {
    Unit,
    Bool(bool),
    Num(f64),
    Str(&'src str),
}

#[derive(Clone, Debug)]
pub enum UnaryOp { Neg, Not }

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Rem,
    And, Or,
    Eq, Ne, Lt, Le, Gt, Ge,
}

pub type Spanned<T> = (T, Span);
type Binding<'src> =
    (&'src str, Option<Type>, Spanned<Box<Expr<'src>>>);

// Clone is needed for type checking since the type checking
// algorithm is recursive and sometimes consume the AST.
#[derive(Clone, Debug)]
pub enum Expr<'src> {
    Lit(Lit<'src>),
    Ident(&'src str),

    Unary(UnaryOp, Spanned<Box<Self>>),
    Binary(BinaryOp, Spanned<Box<Self>>, Spanned<Box<Self>>),

    Lambda(Vec<(&'src str, Option<Type>)>, Spanned<Box<Self>>),
    Call(Spanned<Box<Self>>, Vec<Spanned<Self>>),

    If {
        cond: Spanned<Box<Self>>,
        t: Spanned<Box<Self>>,
        f: Spanned<Box<Self>>,
    },
    Let {
        bindings: Vec<Binding<'src>>,
        body: Spanned<Box<Self>>,
    },
    Assign(Vec<Binding<'src>>),
    Block {
        exprs: Vec<Spanned<Box<Self>>>,
    },
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
            Token::Num(n)  => Expr::Lit(Lit::Num(n)),
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

        let lambda = just(Token::Lambda)
            .ignore_then(
                (
                    symbol
                        .then(
                            just(Token::Colon)
                                .ignore_then(type_parser())
                                .or_not())
                )
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
            )
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .map(|(args, body)| Expr::Lambda(args, boxspan(body)));

        // (ident (: type)?)*
        let binds = symbol
            .then(
                just(Token::Colon)
                    .ignore_then(type_parser())
                    .or_not()
            )
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|((name, ty), expr)| (name, ty, boxspan(expr)))
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>();

        let let_or_assign = just(Token::Let)
            .ignore_then(binds)
            .then(
                just(Token::In)
                    .ignore_then(expr.clone())
                    .or_not()
            )
            .map(|(bindings, body)| match body {
                Some(body) => Expr::Let { bindings, body: boxspan(body) },
                None => Expr::Assign(bindings),
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
            .separated_by(just(Token::Semicolon))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just(Token::Open(Delim::Brace)),
                just(Token::Close(Delim::Brace)),
            )
            .map(|exprs| Expr::Block { exprs });

        let atom = lit
            .or(ident)
            .or(paren_expr)
            .or(lambda)
            .or(let_or_assign)
            .or(if_)
            .or(block)
            .map_with_span(|e, s| (e, s))
            .boxed();

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

        call
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
            Token::Ident("bool") => Type::Bool,
            Token::Ident("num")  => Type::Num,
            Token::Ident("str")  => Type::Str,
            Token::Unit          => Type::Unit,
            Token::Ident(s)      => Type::Var(s.to_string()),
        };

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

        let array = just(Token::Open(Delim::Brack))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Close(Delim::Brack)))
            .map(|t| Type::Array(Box::new(t)));

        lit_ty
            .or(array)
            .or(func)
            .or(tuple)
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