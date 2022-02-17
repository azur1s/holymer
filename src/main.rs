use std::fs;

use chumsky::{Parser, Stream};
use clap::Parser as ArgParser;

/// Arguments handler.
pub mod args;
use args::{Args, Options};

/// Front-end of the language.
/// Contains lexer, parser and token types.
pub mod front;
use front::parse::{lexer, parser};

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile { input: src, ast: _print_ast } => {
            let src = fs::read_to_string(src).expect("Failed to read file");
            let (tokens, lex_error) = lexer().parse_recovery(src.as_str());
            let len = src.chars().count();
            let (ast, parse_error) = parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.clone().unwrap().into_iter()));
            if lex_error.is_empty() {
                if parse_error.is_empty() {
                    println!("{:#?}", ast);
                } else {
                    eprintln!("{:#?}\n(Parser error)", parse_error);
                }
            } else {
                eprintln!("{:#?}\n(Lexer error)", lex_error);
            }
        },
    }
}