use std::{fs, io::Write};

use clap::Parser as ArgParser;

use lexer::lex;
use parser::parse;
use diagnostic::Diagnostics;
use hir::ast_to_ir;
use codegen::cpp;

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
            log: should_log,
            output,
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
                    if diagnostics.has_error() {
                        diagnostics.display(src);
                        logif!(0, "Epic Lowering(HIR) fail");
                        std::process::exit(1);
                    } else {
                        logif!(0, format!("Lowering took {}ms", start.elapsed().as_millis()));
                    }

                    // Generate code
                    let mut codegen = cpp::Codegen::new();
                    codegen.gen(ir);
                    logif!(0, "Successfully generated code.");

                    // Write code to file
                    let output_path = match output {
                        Some(output) => output,
                        None => file_name.with_extension("cpp").file_name().unwrap().to_os_string().into(),
                    };
                    let mut file = fs::File::create(&output_path).expect("Failed to create file");
                    file.write_all(codegen.emitted.as_bytes()).expect("Failed to write to file");

                    // End timer
                    let duration = start.elapsed().as_millis();

                    logif!(0, format!("Compilation took {}ms", duration));
                    logif!(0, format!("Wrote output to `{}`. All done.", output_path.display()));
                },
                None => {
                    unreachable!();
                }
            }
        }
    }
}
