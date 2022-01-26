use std::{fs::{read_to_string, File}, path::Path, io::Write, time::Instant};

use structopt::StructOpt;

mod args;
use args::Args;

mod util;
use util::cover_paren;

mod parser;
use parser::{tokenize, Parser};

mod compiler;
use compiler::compile::Compiler;

fn main() {
    let start = Instant::now();
    let args = Args::from_args();
    
    let src = cover_paren(read_to_string(&args.file).unwrap());
    let file_name = Path::new(&args.file).file_stem().unwrap().to_str().unwrap();
    
    let tokens = tokenize(&src);
    let mut parser = Parser::new(tokens.clone());
    let result = parser.parse();

    match args.verbose {
        0 => {
            let mut file = File::create(format!("{}.bbb", file_name)).unwrap();

            let mut compiler = Compiler::new();
            let before = Instant::now();
            for instr in compiler.compile(result.unwrap(), 0).unwrap() {
                write!(file, "{}\n", instr).unwrap();
            }
            let spent = before.elapsed();
            let total = start.elapsed();

            println!("Compiled in {}.{}s, Total of {}.{}s", spent.as_secs(), spent.subsec_millis(), total.as_secs(), total.subsec_millis());
        },
        1 => {
            println!("Parsed AST: {:#?}", result);

            let mut file = File::create(format!("{}.bbb", file_name)).unwrap();

            let mut compiler = Compiler::new();
            let before = Instant::now();
            for instr in compiler.compile(result.unwrap(), 0).unwrap() {
                write!(file, "{}\n", instr).unwrap();
            }
            let spent = before.elapsed();
            let total = start.elapsed();

            println!("Compiled in {}.{}s, Total of {}.{}s", spent.as_secs(), spent.subsec_millis(), total.as_secs(), total.subsec_millis());
        },
        2 | _ => {
            println!("Tokens: {:?}", tokens);
            println!("Parsed AST: {:#?}", result);

            let mut file = File::create(format!("{}.bbb", file_name)).unwrap();

            let mut compiler = Compiler::new();
            let before = Instant::now();
            for instr in compiler.compile(result.unwrap(), 0).unwrap() {
                write!(file, "{}\n", instr).unwrap();
            }
            let spent = before.elapsed();
            let total = start.elapsed();

            println!("Compiled in {}.{}s, Total of {}.{}s", spent.as_secs(), spent.subsec_millis(), total.as_secs(), total.subsec_millis());
        }
    }
}
