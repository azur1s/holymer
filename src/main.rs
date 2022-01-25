use std::{fs::{read_to_string, File}, path::Path, io::Write};

use structopt::StructOpt;

mod args;
use args::Args;

mod util;
use util::cover_paren;

mod parser;
use parser::{tokenize, Parser};

mod compile;
use compile::Compiler;

fn main() {
    let args = Args::from_args();
    
    let src = cover_paren(read_to_string(&args.file).unwrap());
    let file_name = Path::new(&args.file).file_stem().unwrap().to_str().unwrap();
    
    let tokens = tokenize(&src);
    let mut parser = Parser::new(tokens.clone());
    let result = parser.parse();

    match args.verbose {
        0 => {
            let mut compiler = Compiler::new();
            let instrs = compiler.compile_sexpr(result.unwrap());
            
            let mut file = File::create(format!("{}.bbb", file_name)).unwrap();
            for instr in instrs {
                file.write_all(format!("{}\n", instr).as_bytes()).unwrap();
            }
        },
        1 => println!("{:?}", result),
        2 | _ => {
            println!("Tokens: {:?}", tokens);
            println!("Parsed: {:#?}", result);
        }
    }
}
