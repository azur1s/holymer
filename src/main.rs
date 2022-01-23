use std::{fs::read_to_string, env::args, path::Path};

mod util;
use util::cover_paren;

mod parser;
use parser::tokenize;

fn main() {
    let args = args().nth(1).expect("No input file");
    
    let src = cover_paren(read_to_string(&args).unwrap());
    let _file_name = Path::new(&args).file_stem().unwrap().to_str().unwrap();
    
    let mut parser = parser::Parser::new(tokenize(&src));
    let result = parser.parse();
    println!("{:#?}", result);
}
