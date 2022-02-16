use chumsky::prelude::*;

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Ident(String),
    Unary { op: String, expr: Box<Self> },
    Binary { op: String, left: Box<Self>, right: Box<Self> },

    Let {
        name: String,
        value: Box<Self>,
    },
    Fun {
        name: String,
        args: Vec<String>,
        body: Box<Self>,
    },
    Call {
        name: String,
        args: Vec<Self>,
    },
}

fn expr_parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let ident = text::ident().padded();

    let expr = recursive(|expr| {
        let int = text::int(10)
            .map(|s: String| Expr::Int(s.parse().unwrap()));
        
        let float = text::int(10)
            .then_ignore(just('.'))
            .chain::<char, _, _>(text::digits(10))
            .collect::<String>()
            .map(|s: String| Expr::Float(s.parse().unwrap()));

        let call = ident
            .then(expr.clone()
                .separated_by(just(','))
                .allow_trailing()
                .delimited_by(just('('), just(')')))
            .map(|(name, args)| Expr::Call { name, args });

        let atom = int
            .or(float)
            .or(call)
            .or(ident.map(Expr::Ident))
            .or(expr.delimited_by(just('('), just(')')))
            .labelled("atom");
        
        let unary =  choice((just('-'), just('!')))
            .repeated()
            .then(atom)
            .foldr(|op, rhs| Expr::Unary { op: op.to_string(), expr: Box::new(rhs) }).labelled("unary");
        
        let factor = unary.clone()
            .then(choice((just('*'), just('/')))
                .then(unary)
                .repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op: op.to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs)
            }).labelled("factor");
        
        let term = factor.clone()
            .then(choice((just('+'), just('-')))
                .then(factor)
                .repeated())
            .foldl(|lhs, (op, rhs)| Expr::Binary {
                op: op.to_string(),
                left: Box::new(lhs),
                right: Box::new(rhs)
            }).labelled("term");
        
        term.padded()
    }).labelled("expression");

    let declare = recursive(|decl| {
        let declare_var = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .map(|(name, rhs)| Expr::Let {
                name,
                value: Box::new(rhs),
            });

        let declare_fun = text::keyword("fun")
            .ignore_then(ident)
            .then(ident.repeated())
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .map(|((name, args), body)| Expr::Fun {
                name,
                args,
                body: Box::new(body),
            });

        declare_var
            .or(declare_fun)
            .or(expr)
            .padded()
    });

    declare
}

pub fn parser() -> impl Parser<char, Vec<Expr>, Error = Simple<char>> {
    expr_parser()
        .repeated()
        .then_ignore(end())
}