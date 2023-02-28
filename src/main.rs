#![feature(trait_alias)]
pub mod read;
pub mod trans;

use read::parse::{lex, parse};
use trans::low::{translate_expr, translate_js};

fn main() {
    let path = std::env::args().nth(1).expect("No file path provided");
    let src = std::fs::read_to_string(path).expect("Failed to read file");

    let (tokens, lex_errs) = lex(src.to_owned());

    let parse_errs = if let Some(tokens) = tokens {
        let (ast, parse_errs) = parse(tokens, src.len());

        if let Some(ast) = ast {
            println!();
            println!("\x1b[90m───SOURCE─────────────────────────────────────────\x1b[0m");
            println!("{src}");
            println!("\x1b[90m───PARSE TREE─────────────────────────────────────\x1b[0m");
            for (e, _) in &ast {
                println!("{}", {
                    let e = format!("{:?}", e);
                    if e.len() > 50 {
                        format!("{}...", &e[..47])
                    } else {
                        e
                    }
                });
            }
            println!("\x1b[90m───INTERNAL AST───────────────────────────────────\x1b[0m");
            let nexprs = ast.into_iter().map(|(e, _)| translate_expr(e)).collect::<Vec<_>>();

            for expr in &nexprs {
                println!("{}", expr);
            }
            println!("\x1b[90m───JS OUTPUT──────────────────────────────────────\x1b[0m");
            let jsexprs = nexprs.into_iter().map(translate_js).collect::<Vec<_>>();

            for expr in &jsexprs {
                println!("{}", expr);
            }
            println!();
        }

        parse_errs
    } else {
        Vec::new()
    };

    if !lex_errs.is_empty() || !parse_errs.is_empty() {
        lex_errs
            .into_iter()
            .map(|e| e.map(|c| c.to_string()))
            .chain(parse_errs.into_iter().map(|e| e.map(|t| t.to_string())))
            .for_each(|e| println!("{}", e));
    }
}
