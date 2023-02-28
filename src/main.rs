#![feature(trait_alias)]
pub mod parse;
pub mod trans;

use parse::parse::lex;

fn main() {
    let input = r#"
        println((\x: int -> x + 1)(1));
    "#;

    let tokens = lex(input.to_owned());
    println!("{:?}", tokens);

    // use parse::past::*;
    // use trans::ty::Type;
    // use trans::low::*;

    // let exprs = vec![
    //     PExpr::Call(Box::new(PExpr::Sym("println".to_string())), vec![
    //         PExpr::Str("Hello, world!".to_string()),
    //     ]),
    //     PExpr::Let {
    //         vars: vec![
    //             ("x".to_string(), Type::Num, PExpr::Num(1)),
    //         ],
    //         body: Box::new(PExpr::Sym("x".to_string())),
    //     },
    //     PExpr::Let {
    //         vars: vec![
    //             ("x".to_string(), Type::Num, PExpr::Num(34)),
    //             ("y".to_string(), Type::Num, PExpr::Num(35)),
    //         ],
    //         body: Box::new(PExpr::BinaryOp(
    //             PBinaryOp::Add,
    //             Box::new(PExpr::Sym("x".to_string())),
    //             Box::new(PExpr::Sym("y".to_string())),
    //         )),
    //     },
    // ];

    // let nexprs = exprs.into_iter().map(translate_expr).collect::<Vec<_>>();

    // for expr in &nexprs {
    //     println!("{}", expr);
    // }

    // println!("──────────────────────────────────────────────────");

    // let jsexprs = nexprs.into_iter().map(translate_js).collect::<Vec<_>>();

    // for expr in &jsexprs {
    //     println!("{}", expr);
    // }
}
