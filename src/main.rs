use std::fs;

use chumsky::Parser;
use clap::Parser as ArgParser;

/// Arguments handler.
pub mod args;
use args::{Args, Options};

/// Front-end of the language.
/// Contains lexer, parser and token types.
pub mod front;
use front::parse::parser;

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile { input: src, ast: _print_ast } => {
            let src = fs::read_to_string(src).expect("Failed to read file");
            let tokens = parser().parse_recovery(src.as_str());
            println!("{:?}", tokens);
        },
    }
}