use std::{fs::{read_to_string, File}, path::Path, io::{Write, Seek}, time::Instant};

use structopt::StructOpt;

mod args;
use args::Args;

mod util;
use util::cover_paren;

mod parser;
use parser::{tokenize, Parser, Sexpr};

mod compiler;
use compiler::{compile::Compiler, instr::Instr};

mod vm;

fn main() {
    let start = Instant::now();
    let args = Args::from_args();
    
    let src = cover_paren(read_to_string(&args.file).unwrap());
    let file_name = match args.output {
        Some(path) => path,
        None => Path::new(&args.file).to_path_buf(),
    }.file_stem().unwrap().to_str().unwrap().to_string();
    
    let tokens = tokenize(&src);
    let mut parser = Parser::new(tokens.clone());
    let result = parser.parse();

    match result {
        Ok(ast) => {
            compile(ast, file_name, start);
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }

}

fn compile(ast: Sexpr, file_name: String, start: Instant) {
    let mut compiler = Compiler::new();
    let code = compiler.compile(ast, 0);
    match code {
        Ok(code) => {
            let mut file = File::create(format!("{}.bsm", file_name)).unwrap();
            for line in code {
                write!(file, "{}\n", line).unwrap();
            }
            file.seek(std::io::SeekFrom::End(-1)).unwrap(); // Trim last newline

            let elapsed = start.elapsed();
            println!("Compiled in {}.{}s", elapsed.as_secs(), elapsed.subsec_millis());
        },
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
