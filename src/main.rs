use std::{process::exit, fs::read_to_string};

pub mod parser;
pub mod compile;

const EXECUTABLE_NAME: &str = env!("CARGO_PKG_NAME");
const HELP_MESSAGE: &str = "\
    -h, --help
        Print this help message and exit.
    -v, --version
        Print version information and exit.
    -c FILE, --compile FILE
        Compile the given file and exit.\
";

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut args_index: usize = 0;
    match args.len() {
        // No argument provided
        1 => {
            println!("No argument provided.");
            display_help(1);
        },
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
                            // Used for error reporting
                            let file_content_joined: String = file_content.split("\n").collect::<Vec<&str>>().join(" ");

                            let parsed = parser::parse(&file_content);
                            let mut ast = Vec::new();
                            for node in parsed {
                                match node {
                                    Ok(node) => { ast.push(node); },
                                    Err(error) => {
                                        eprintln!("{}", error.at(&file_content_joined));
                                        exit(1);
                                    }
                                }
                            }
                            println!("{:#?}", ast);
                        } else {
                            println!("No file provided.");
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
    println!("Usage: {} [OPTIONS]", EXECUTABLE_NAME);
    println!("{}", HELP_MESSAGE);
    exit(exit_code);
}