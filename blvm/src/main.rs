use std::{fs::read_to_string, path::Path};
use structopt::StructOpt;

mod args;
use args::Args;

mod parser;
use parser::tokenize;

fn main() {
    let args = Args::from_args();
    
    let src = read_to_string(&args.file).unwrap();
    let _file_name = Path::new(&args.file).file_stem().unwrap().to_str().unwrap();

    let tokens = tokenize(&src);
}
