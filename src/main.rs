#![feature(trait_alias)]
pub mod asts;
pub mod read;
pub mod trans;
pub mod args;

use std::io::Write;
use args::Options;
use read::parse::{lex, parse};
use structopt::StructOpt;
use trans::low::{translate_stmt, translate_js_stmt};

fn main() {
    let opt = Options::from_args();
    let src = std::fs::read_to_string(opt.file).expect("Failed to read file");

    let (tokens, lex_errs) = lex(src.to_owned());

    let parse_errs = if let Some(tokens) = tokens {
        let (past, parse_errs) = parse(tokens, src.len());

        if let Some(past) = past {
            let ast = past.into_iter().map(|(e, _)| translate_stmt(e)).collect::<Vec<_>>();
            let js = ast.into_iter().map(translate_js_stmt).collect::<Vec<_>>();

            let mut file = std::fs::File::create(opt.output.unwrap_or("out.js".into()))
                .expect("Failed to create file");
            let s = js
                .into_iter()
                .map(|e| {
                    let s = format!("{}", e);
                    if s.ends_with(';') {
                        s
                    } else {
                        format!("{};", s)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            file.write_all(s.as_bytes()).expect("Failed to write to file");
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
            .for_each(|e| println!("[{:?} {:?}] {}", e.span(), e.label(), e));
    }
}
