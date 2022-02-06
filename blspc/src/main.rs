use std::{fs::{read_to_string, File}, path::{Path, PathBuf}, io::{Write, BufWriter}, time::Instant, process::exit};

use structopt::StructOpt;

mod args;
use args::Opts;

mod util;
use util::cover_paren;

mod compiler;
use compiler::{compile::Compiler, parser::{tokenize, Parser}};

mod vm;
use vm::{vm::VM, parser::parse_instr};

fn main() {
    let args = Opts::from_args();
    match args.commands {
        args::Args::Compile(args) => {
            let src = read_to_string(&args.file).unwrap();
            let debug = args.debug;
            compile_src(src, args.output, args.file, debug);
        },
        args::Args::Run(args) => {
            let src = read_to_string(&args.file).unwrap();
            let debug = args.debug;
            run_src(src, debug);
        },
    }
}

fn compile_src(src: String, path: Option<PathBuf>, file: PathBuf, debug: bool) {
    let file_name = match path {
        Some(path) => path,
        None => Path::new(&file).to_path_buf(),
    }.file_stem().unwrap().to_str().unwrap().to_string();
    
    let start = Instant::now();
    let tokens = tokenize(&cover_paren(src));
    let mut parser = Parser::new(tokens.clone());
    let result = parser.parse();
    
    if debug { println!("{:#?}", &result); }
    match result {
        Ok(ast) => {
            let mut compiler = Compiler::new();
            let code = compiler.compile(ast);
            match code {
                Ok(code) => {
                    let file = File::create(format!("{}.bsm", file_name)).unwrap();
                    let mut buffer = BufWriter::new(file);
                    for line in code {
                        writeln!(buffer, "{}", line).unwrap();
                    }
                    buffer.flush().unwrap();
                    
                    let elapsed = start.elapsed();
                    println!("Compiled in {}.{}s", elapsed.as_secs(), elapsed.subsec_millis());
                },
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn run_src(src: String, debug: bool) {
    let instrs = parse_instr(&src);
    let mut vm = VM::new();
    match vm.run(instrs, debug) {
        Ok(()) => {
            exit(0);
        },
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
