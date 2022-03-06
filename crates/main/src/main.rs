use std::fs;

use clap::Parser as ArgParser;
use lexer::lex;
use parser::parse;

pub mod args;
use args::{Args, Options};

pub mod util;
use crate::util::log;

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile {
            input: file_name,
            ast: _print_ast,
        } => {
            // Get file contents.
            let src = fs::read_to_string(&file_name).expect("Failed to read file");

            // Lex the file.
            let (tokens, lex_error) = lex(src.clone());
            
            if lex_error.is_empty() {
                log(0, "Lexing successful.");

                let (ast, parse_error) = parse(tokens.unwrap(), src.chars().count());

                if parse_error.is_empty() {
                    println!("{:#?}", ast);
                    log(0, "Parsing successful.");
                } else {
                    println!("{:#?}", parse_error);
                    log(2, "Parsing failed.");
                }
            } else {
                println!("{:#?}", lex_error);
                log(2, "Lexing failed.");
            }
        }
    }
}
