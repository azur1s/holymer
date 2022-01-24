use std::{fs::read_to_string, path::Path};

use structopt::StructOpt;

mod args;
use args::Args;

mod util;
use util::cover_paren;

mod parser;
use parser::{tokenize, Parser};

fn main() {
    let args = Args::from_args();
    
    let src = cover_paren(read_to_string(&args.file).unwrap());
    let _file_name = Path::new(&args.file).file_stem().unwrap().to_str().unwrap();
    
    let tokens = tokenize(&src);
    let mut parser = Parser::new(tokens.clone());
    let result = parser.parse();

    match args.verbose {
        0 => println!("We don't do anything yet."),
        1 => println!("{:?}", result),
        2 | _ => {
            println!("Tokens: {:?}", tokens);
            println!("Parsed: {:#?}", result);
        }
    }
}
