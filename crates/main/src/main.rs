use std::{fs, io::Write, process::Command, path::PathBuf};

use clap::Parser as ArgParser;

use lexer::lex;
use parser::parse;
use diagnostic::Diagnostics;
use hir::ast_to_ir;
use typecheck::check;
use codegen::ts;

pub mod args;
use args::{Args, Options};

pub mod util;
use crate::util::log;

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile {
            input: file_name,
            ast: print_ast,
            log: should_log,
            output: _output, // TODO: Custom output file
        } => {
            // Macro to only log if `should_log` is true
            macro_rules! logif {
                ($level:expr, $msg:expr) => { if should_log { log($level, $msg); } };
            }

            // Start timer
            let start = std::time::Instant::now();

            // Get file contents
            logif!(0, format!("Reading {}", &file_name.display()));
            let src = fs::read_to_string(&file_name).expect("Failed to read file");

            // Lex the file
            let (tokens, lex_error) = lex(src.clone());
            let (ast, parse_error) = parse(tokens.unwrap(), src.chars().count());

            let mut diagnostics = Diagnostics::new();
            for err in lex_error   { diagnostics.add_lex_error(err);   }
            for err in parse_error { diagnostics.add_parse_error(err); }

            // Report syntax errors if any
            if diagnostics.has_error() {
                diagnostics.display(src);
                logif!(0, "Epic parsing fail");
                std::process::exit(1);
            } else {
                logif!(0, format!("Parsing took {}ms", start.elapsed().as_millis()));
            }

            match ast {
                Some(ast) => {
                    // Convert the AST to HIR
                    let (ir, lowering_error) = ast_to_ir(ast);
                    for err in lowering_error { diagnostics.add_lowering_error(err); }

                    if print_ast { log(0, format!("IR\n{:#?}", ir)); }

                    // Typecheck the HIR
                    match check(&ir) {
                        Ok(_) => {
                            logif!(0, format!("Typechecking took {}ms", start.elapsed().as_millis()));
                        },
                        Err(errs) => {
                            for err in errs {
                                diagnostics.add_typecheck_error(err);
                            }
                            diagnostics.display(src);
                            logif!(2, "Typechecking failed");
                            std::process::exit(1);
                        }
                    }

                    // Report lowering errors if any
                    if diagnostics.has_error() {
                        diagnostics.display(src);
                        logif!(0, "Epic Lowering(HIR) fail");
                        std::process::exit(1);
                    } else {
                        logif!(0, format!("Lowering took {}ms", start.elapsed().as_millis()));
                    }

                    // Generate code
                    let mut codegen = ts::Codegen::new();
                    codegen.gen(ir);
                    logif!(0, "Successfully generated code.");

                    // Write code to file
                    let output_path: PathBuf = file_name.with_extension("ts").file_name().unwrap().to_os_string().into();
                    let mut file = fs::File::create(&output_path).expect("Failed to create file");
                    file.write_all(codegen.emitted.as_bytes()).expect("Failed to write to file");

                    // End timer
                    let duration = start.elapsed().as_millis();

                    logif!(0, format!("Compilation took {}ms", duration));
                    logif!(0, format!("Wrote output to `{}`", output_path.display()));
                },
                None => { unreachable!(); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let src = "
            let x: int = 1;
        ";

        let (tokens, lex_error) = lex(src.to_string());
        assert!(lex_error.is_empty());

        assert_eq!(tokens.unwrap().len(), 7);
    }

    #[test]
    fn test_parser() {
        let src = "
            fun main (foo: int) (bar: bool): string = do
                do
                    let x: int = foo + 1;
                end;
                let y: bool = bar;
            end;
        ";

        let (tokens, lex_error) = lex(src.to_string());
        assert!(lex_error.is_empty());

        let (ast, parse_error) = parse(tokens.unwrap(), src.chars().count());
        assert!(parse_error.is_empty());

        assert!(ast.is_some());
    }
}