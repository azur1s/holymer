use super::{*, ast::*, lex::{Token, Delimiter}};
use chumsky::{prelude::*, Stream};

pub trait P<T> = chumsky::Parser<Token, T, Error = Simple<Token>> + Clone;

fn identifier() -> impl P<Spanned<String>> {
    filter_map(|span, token| match token {
        Token::Identifier(s) => Ok((s, span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    }).labelled("identifier")
}

fn literal() -> impl P<Spanned<Literal>> {
    filter_map(|span, token| match token {
        Token::Int(i)     => Ok((ast::Literal::Int(i), span)),
        Token::Boolean(b) => Ok((ast::Literal::Boolean(b), span)),
        Token::String(s)  => Ok((ast::Literal::String(s), span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    }).labelled("literal")
}

fn typehint_parser() -> impl P<Spanned<Typehint>> {
    recursive(|ty| {

    let single = filter_map(|span, token| match token {
        Token::Identifier(s) => Ok((
            match s.as_str() {
                "any"       => ast::Typehint::Builtin(ast::BuiltinType::Any),
                "null"      => ast::Typehint::Builtin(ast::BuiltinType::Null),
                "undefined" => ast::Typehint::Builtin(ast::BuiltinType::Undefined),
                "bool"      => ast::Typehint::Builtin(ast::BuiltinType::Boolean),
                "int"       => ast::Typehint::Builtin(ast::BuiltinType::Int),
                "string"    => ast::Typehint::Builtin(ast::BuiltinType::String),
                _ => Typehint::Single(s),
            }, span)),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(token))),
    });

    let tuple = single
    .separated_by(just(Token::Comma))
    .allow_trailing()
    .delimited_by(
        just(Token::Open(Delimiter::Paren)),
        just(Token::Close(Delimiter::Paren)))
    .map_with_span(|args, span| {( Typehint::Tuple(args), span )});

    let vector = single
    .delimited_by(
        just(Token::Open(Delimiter::Bracket)),
        just(Token::Close(Delimiter::Bracket)))
    .map_with_span(|arg, span| {( Typehint::Vector(Box::new(arg)), span )});

    let function = ty.clone()
    .separated_by(just(Token::Comma))
    .allow_trailing()
    .delimited_by(
        just(Token::Or),
        just(Token::Or))
    .then_ignore(just(Token::Arrow))
    .then(ty.clone())
    .map_with_span(|(args, ret), span| {( Typehint::Function(args, Box::new(ret)), span )});

    let union_ty = ty.clone()
    .separated_by(just(Token::Or))
    .allow_trailing()
    .delimited_by(
        just(Token::Open(Delimiter::Paren)),
        just(Token::Close(Delimiter::Paren)))
    .map_with_span(|args, span| {( Typehint::Union(args), span )});

    single
        .or(tuple)
        .or(vector)
        .or(function)
        .or(union_ty)
    })
}

fn expr_parser() -> impl P<Spanned<Expr>> {
    recursive(|expr| {

        // Atom ::= Literal
        //        | Identifier
        //        | Vector
        //        | Tuple
        //        | Object

        let args = expr.clone().separated_by(just(Token::Comma)).allow_trailing();

        let vec = args.clone().delimited_by(
            just(Token::Open(Delimiter::Bracket)),
            just(Token::Close(Delimiter::Bracket)))
        .map_with_span(|args, span| {( Expr::Vector(args), span )});

        let tuple = args.clone().delimited_by(
            just(Token::Open(Delimiter::Paren)),
            just(Token::Close(Delimiter::Paren)))
        .map_with_span(|args, span| {( Expr::Tuple(args), span )});

        let object = identifier()
        .then_ignore(just(Token::Colon))
        .then(expr.clone())
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(
            just(Token::Open(Delimiter::Brace)),
            just(Token::Close(Delimiter::Brace)))
        .map_with_span(|args, span| {( Expr::Object { fields: args }, span )});

        let atom = literal().map_with_span(|literal, span| {( Expr::Literal(literal), span )})
        .or(identifier().map_with_span(|ident, span| {( Expr::Identifier(ident), span )}))
        .or(vec)
        .or(tuple)
        .or(object)
        .labelled("atom");

        // Call      ::= Identifier '(' ( Expr ( ',' Expr )* )? ')'
        // Method    ::= Identifier '.' Identifier ( '(' ( Expr ( ',' Expr )* )? ')' )
        // Access    ::= Identifier '.' Idnetifier
        // Intrinsic ::= '@' Call
        // Unary     ::= UnaryOp ( Call | Intrinsic )
        // Binary    ::= Unary BinaryOp Unary

        let identexpr = identifier().map_with_span(|ident, span| {( Expr::Identifier(ident), span )});

        let call = atom.clone()
        .then(
            args.clone().delimited_by(
                    just(Token::Open(Delimiter::Paren)),
                    just(Token::Close(Delimiter::Paren)))
            .repeated())
        .foldl(|name, args| {( Expr::Call { name: Box::new(name.clone()), args }, name.1 )}).labelled("call");

        let intrinsic = just(Token::At).ignore_then(atom.clone())
        .then(
            args.clone().delimited_by(
                    just(Token::Open(Delimiter::Paren)),
                    just(Token::Close(Delimiter::Paren)))
            .repeated())
        .foldl(|name, args| {( Expr::Intrinsic { name: Box::new(name.clone()), args }, name.1 )}).labelled("intrinsic");

        let method = just(Token::Colon)
        .ignore_then(identexpr.clone())
        .then_ignore(just(Token::Dot))
        .then(atom.clone())
        .then(
            args.clone().delimited_by(
                    just(Token::Open(Delimiter::Paren)),
                    just(Token::Close(Delimiter::Paren)))
            .repeated())
        .map_with_span(|((name, method), args), span| {( Expr::Method {
            obj: Box::new(name),
            name: Box::new(method),
            args: args.into_iter().flatten().collect()
        }, span )}).labelled("method");

        let access = just(Token::Semicolon)
        .ignore_then(identexpr)
        .then_ignore(just(Token::Dot))
        .then(atom.clone())
        .map_with_span(|(obj, name), span| {( Expr::Access { obj: Box::new(obj), name: Box::new(name) }, span )}).labelled("access");

        let unary = choice((
            just(Token::Minus).to(UnaryOp::Minus),
            just(Token::Not).to(UnaryOp::Not)))
        .repeated()
        .then(call.or(intrinsic).or(method).or(access))
        .foldr(|op, rhs| {( Expr::Unary { op, rhs: Box::new(rhs.clone()) }, rhs.1 )});

        let factor = unary.clone().then(
            choice((
                just(Token::Multiply).to(BinaryOp::Multiply),
                just(Token::Divide).to(BinaryOp::Divide),
                just(Token::Modulus).to(BinaryOp::Modulus)))
            .then(unary)
            .repeated())
        .foldl(|lhs, (op, rhs)| {(
            Expr::Binary {
                lhs: Box::new(lhs), op, rhs: Box::new(rhs.clone()),
            }, rhs.1)});

        let term = factor.clone().then(
            choice((
                just(Token::Plus).to(BinaryOp::Plus),
                just(Token::Minus).to(BinaryOp::Minus)))
            .then(factor)
            .repeated())
        .foldl(|lhs, (op, rhs)| {(
            Expr::Binary {
                lhs: Box::new(lhs), op, rhs: Box::new(rhs.clone()),
            }, rhs.1)});

        let compare = term.clone().then(
            choice((
                just(Token::Equal).to(BinaryOp::Equal),
                just(Token::NotEqual).to(BinaryOp::NotEqual),
                just(Token::Less).to(BinaryOp::Less),
                just(Token::Greater).to(BinaryOp::Greater)))
            .then(term)
            .repeated())
        .foldl(|lhs, (op, rhs)| {(
            Expr::Binary {
                lhs: Box::new(lhs), op, rhs: Box::new(rhs.clone()),
            }, rhs.1)});

        // Do       ::= 'do' Expr* 'end'
        // Define   ::= Identifier ':' Typehint '=' Expr
        // Redefine ::= 'set' Identifier '=' Expr
        // Function ::= 'fun' Identifier ( Identifier* ) '(' ( Identifier ':' Typehint ( ',' Identifier ':' Typehint )* )? ')' ':' Typehint '=' Expr
        // If       ::= 'if' Expr '|' Expr '|' Expr
        // Return   ::= 'return' Expr
        // Note: This section's `Expr` might actually mean `Expr | Do`

        let do_block = expr.clone().repeated()
        .delimited_by(
            just(Token::KwDo),
            just(Token::KwEnd))
        .map_with_span(|body, span| {( Expr::Do {body: (body, span.clone())}, span )});

        let define = identifier()
        // Type hint
        .then(just(Token::Colon).ignore_then(typehint_parser()))
        // Body
        .then(just(Token::Assign).ignore_then(do_block.clone().or(expr.clone())))
        .map_with_span(|((ident, typehint), expr), span| {
            (Expr::Define { name: *Box::new(ident), typehint, value: Box::new(expr) }, span)
        });

        let redefine = just(Token::KwSet)
        .ignore_then(identifier())
        // Body
        .then(just(Token::Assign).ignore_then(do_block.clone().or(expr.clone())))
        .map_with_span(|(ident, expr), span| {
            (Expr::Redefine { name: *Box::new(ident), value: Box::new(expr) }, span)
        });

        let function = just(Token::KwFun)
        .ignore_then(identifier())
        // Generics
        .then(identifier().repeated())
        // Arguments
        .then(
            identifier()
                .then_ignore(just(Token::Colon))
                .then(typehint_parser())
                .delimited_by(
                    just(Token::Open(Delimiter::Paren)),
                    just(Token::Close(Delimiter::Paren)))
                .repeated())
        // Return type hint
        .then_ignore(just(Token::Colon))
        .then(typehint_parser())
        // Body
        .then_ignore(just(Token::Assign))
        .then(do_block.clone().or(expr.clone()))
        .map(|((((name, generics), args), typehint), body)| {
            ( Expr::Function {
                name: *Box::new(name),
                generics,
                args: args.into_iter().map(|(name, typehint)| {
                    (name, *Box::new(typehint))
                }).collect(),
                typehint,
                body: Box::new(body.clone()) }, body.1 )});

        let if_else = just(Token::KwIf)
        // Condition
        .ignore_then(expr.clone())
        // True branch
        .then_ignore(just(Token::Or))
        .then(do_block.clone().or(expr.clone()))
        // False branch
        .then_ignore(just(Token::Or))
        .then(do_block.clone().or(expr.clone()))
        .map_with_span(|((cond, then), else_), span| {
            (Expr::If { cond: Box::new(cond), t: Box::new(then), f: Box::new(else_) }, span)
        });

        let return_ = just(Token::KwReturn)
        .ignore_then(expr.clone())
        .map_with_span(|expr, span| {( Expr::Return(Box::new(expr)), span )});

        // Expr ::= Define
        //        | Redefine
        //        | Function
        //        | Do
        //        | Return
        //        | If
        //        | Binary

        define
            .or(redefine)
            .or(function)
            .or(do_block)
            .or(return_)
            .or(if_else)
            .or(compare)
    }).labelled("expression")
}

#[allow(clippy::type_complexity)]
pub fn parse(tokens: Vec<(Token, std::ops::Range<usize>)>, len: usize) -> (Option<Vec<(Expr, std::ops::Range<usize>)>>, Vec<Simple<Token>>) {
    let (ast, parse_error) = expr_parser().repeated().then_ignore(end()).parse_recovery(Stream::from_iter(
        len..len + 1,
        tokens.into_iter(),
    ));

    (ast, parse_error)
}