use std::{fs, io::Write};

use clap::Parser as ArgParser;
use ariadne::{Report, ReportKind, Label, Source, Color, Fmt};

use lexer::lex;
use parser::parse;
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
        } => {
            // Start timer
            let start = std::time::Instant::now();

            // Get file contents
            let src = fs::read_to_string(&file_name).expect("Failed to read file");

            // Lex the file
            let (tokens, lex_error) = lex(src.clone());
            let (ast, parse_error) = parse(tokens.unwrap(), src.chars().count());

            // Report errors
            lex_error.into_iter()
                .map(|e| e.map(|e| e.to_string()))
                .chain(parse_error.into_iter().map(|e| e.map(|tok| tok.to_string())))
                .for_each(|e| {
                    let report = Report::build(ReportKind::Error, (), e.span().start);

                    let report = match e.reason() {
                        chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_label(
                                Label::new(span.clone())
                                    .with_message(format!(
                                        "Expected closing delimiter {}",
                                        delimiter.fg(Color::Yellow)
                                    ))
                                    .with_color(Color::Yellow)
                            )
                            .with_label(
                                Label::new(e.span())
                                    .with_message(format!(
                                        "Must be closed before this {}",
                                        e.found()
                                            .unwrap_or(&"end of file".to_string())
                                            .fg(Color::Red)
                                    ))
                                    .with_color(Color::Red)
                            ),

                        chumsky::error::SimpleReason::Unexpected => report
                            .with_message(format!(
                                "{}, expected {}",

                                if e.found().is_some() { "Unexpected token in input" }
                                else { "Unexpected end of input" },

                                if e.expected().len() == 0 { "something else".to_string().fg(Color::Green) }
                                else {
                                    e.expected()
                                        .map(|expected| match expected {
                                            Some(expected) => expected.to_string(),
                                            None => "end of input".to_string()
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                        .fg(Color::Green)
                                }
                            ))
                            .with_label(
                                Label::new(e.span())
                                    .with_message(format!(
                                        "Unexpected token {}",
                                        e.found()
                                            .unwrap_or(&"EOF".to_string())
                                            .fg(Color::Red)
                                    ))
                                    .with_color(Color::Red)
                            ),
                        _ => {
                            println!("{:?}", e);
                            todo!();
                        }
                    };

                    report.finish().print(Source::from(&src)).unwrap();
                }
            ); // End errors reporting
            log(0, format!("Parsing took {}ms", start.elapsed().as_millis()));

            match ast {
                Some(ast) => {
                    // Convert the AST to HIR
                    let ir = ast_to_ir(ast);
                    log(0, "Generated HIR.");
                    
                    // Generate code
                    let mut codegen = cpp::Codegen::new();
                    codegen.gen(ir);
                    log(0, "Successfully generated code.");

                    // Write code to file
                    let mut file = fs::File::create("out.cpp").expect("Failed to create file");
                    file.write_all(codegen.emitted.as_bytes()).expect("Failed to write to file");

                    // End timer
                    let duration = start.elapsed().as_millis();

                    log(0, format!("Compilation took {}ms", duration));
                    log(0, format!("Wrote output to `out.cpp`. All done."));
                },
                None => {
                    log(2, "Failed to parse.");
                }
            }
        }
    }
}
