use std::fs;

use clap::Parser as ArgParser;

/// Arguments handler.
pub mod args;
use args::{Args, Options};

pub mod front;
use front::lex::Lexer;

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile { input: src, ast: _print_ast } => {
            let bytes: Vec<u8> = fs::read(src).unwrap();
            let tokens = Lexer::lex_tokens(&bytes);
            println!("{:?}", tokens);
        },
    }
}