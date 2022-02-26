use std::{fs, io::Write, time};

use chumsky::{Parser, Stream};
use clap::Parser as ArgParser;

/// Arguments handler.
pub mod args;
use args::{Args, Options};

/// Front-end of the language.
/// Contains lexer, parser and token types.
pub mod front;
use front::parse::{lexer, parser};

/// Middle-end of the language.
/// Contains the intermediate representation.
pub mod middle;
use middle::ir;

/// Back-end of the language.
/// Contains code generator.
pub mod back;

pub mod util;
use crate::util::log;

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile { input: file_name, ast: _print_ast } => {
            // Get file contents.
            let src = fs::read_to_string(&file_name).expect("Failed to read file");
            
            // Lex the file.
            let (tokens, lex_error) = lexer().parse_recovery(src.as_str());
            let len = src.chars().count();
            
            // Parse the file.
            let (ast, parse_error) = parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.clone().unwrap().into_iter()));
            
            if lex_error.is_empty() {

                if parse_error.is_empty() {

                    match ast {
                        // If there is some AST then generate code.
                        Some(ast) => {
                            let start = time::Instant::now();

                            let ir = ir::ast_to_ir(ast);

                            let out = back::js::gen(ir);
                            println!("{}", out);

                            let all_elapsed = start.elapsed();
                            log(0, format!("Done in {}s", all_elapsed.as_secs_f64()));
                        },
                        // If there is no AST, then notify the user.
                        None => println!("no ast :("),
                    };

                } else {
                    eprintln!("{:#?}\n(Parser error)", parse_error);
                }

            } else {
                eprintln!("{:#?}\n(Lexer error)", lex_error);
            }
        },
    }
}