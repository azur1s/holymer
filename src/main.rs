use std::{process::exit, fs::read_to_string};

pub mod parser;

const EXECUTABLE_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut args_index: usize = 0;
    match args.len() {
        // No argument provided
        1 => { display_help(1); },
        _ => {
            while args.len() > args_index {
                let arg: &str = &args[args_index];
                match arg {
                    "-h" | "--help" => { display_help(0); },
                    "-v" | "--version" => {
                        println!("{} version {}", EXECUTABLE_NAME, env!("CARGO_PKG_VERSION"));
                        exit(0);
                    },
                    "-c" | "--compile" => {
                        args_index += 1;
                        if args_index < args.len() {
                            let file_path: &str = &args[args_index];
                            let file_content: String = read_to_string(file_path).unwrap();
                            let ast = parser::parse(&file_content);
                            for node in ast {
                                println!("{:?}", node);
                            }
                        } else {
                            display_help(1);
                        }
                    }
                    _ => { args_index += 1; }
                }
            }
        }
    }
}

fn display_help(exit_code: i32) {
    println!("Usage: {} <file>", EXECUTABLE_NAME);
    exit(exit_code);
}