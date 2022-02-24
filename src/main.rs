use std::{fs, io::Write, time, process::Command};

use chumsky::{Parser, Stream};
use clap::Parser as ArgParser;

/// Arguments handler.
pub mod args;
use args::{Args, Options};

/// Front-end of the language.
/// Contains lexer, parser and token types.
pub mod front;
use front::parse::{lexer, parser};

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

                            let mut compiler = back::c::Codegen::new();
                            compiler.gen(&ast);
                        
                            let out_file_name = file_name.file_stem().unwrap().to_str().unwrap().to_string() + ".c";
                            let mut out_file = fs::File::create(&out_file_name).expect("Failed to create file");
                            write!(out_file, "{}", compiler.emitted).expect("Failed to write to file");
                        
                            let compile_elapsed = start.elapsed();

                            log(0, format!("Compiled successfully to {} in {}s", &out_file_name, compile_elapsed.as_secs_f64()));
                            log(0, "Running clang-format...");
                            let mut clang_format_status = Command::new("clang-format")
                                .arg("-i")
                                .arg(&out_file_name)
                                .spawn()
                                .expect("Failed to run clang-format, make sure you have it installed");
                            match clang_format_status.wait() {
                                Ok(status) => {
                                    match status.success() {
                                        true => log(0, "Successfully run clang-format"),
                                        false => log(2, "Failed to run clang-format"),
                                    }
                                },
                                Err(e) => log(2, format!("Failed to wait on clang-format: {}", e)),
                            }
                            
                            log(0, "Running clang...");
                            let mut clang_status = Command::new("clang")
                                .arg(&out_file_name)
                                .spawn()
                                .expect("Failed to run clang, make sure you have it installed");
                            match clang_status.wait() {
                                Ok(status) => {
                                    match status.success() {
                                        true => log(0, "Successfully run clang"),
                                        false => log(2, "Failed to run clang"),
                                    }
                                },
                                Err(e) => log(2, format!("Failed to wait on clang: {}", e)),
                            }
                            
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