use std::fs;

use clap::Parser as ArgParser;

/// Arguments handler.
pub mod args;
use args::{Args, Options};

pub mod front;
use front::{lex::Lexer, parser::Parser, model::Tokens};

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile { input: src, ast: _print_ast } => {
            let bytes: Vec<u8> = fs::read(src).unwrap();
            let (_errs_, tokens) = Lexer::lex_tokens(&bytes).unwrap();
            let tokens = Tokens::new(&tokens);
            let (_errs_, ast) = Parser::parse(tokens).unwrap();
            println!("{:#?}", ast);
        },
    }
}