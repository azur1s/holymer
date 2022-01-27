use std::{fs::{read_to_string, File}, path::{Path, PathBuf}, io::{Write, Seek}, time::Instant, process::exit};

use structopt::StructOpt;

mod args;
use args::Args;

mod util;
use util::cover_paren;

mod compiler;
use compiler::{compile::Compiler, parser::{tokenize, Parser}};

mod vm;
use vm::{vm::VM, parser::parse_instr};

fn main() {
    let start = Instant::now();
    let args = Args::from_args();

    let debug = match args.verbose {
        0 => false,
        1 => true,
        2 => true,
        _ => true,
    };
    
    match (args.compile, args.run) {
        (true, true) => {
            eprintln!("TODO: Compile and run at the same time.");
            std::process::exit(1);
        },
        // Compile
        (true, false) => {
            let src = read_to_string(&args.file).unwrap();
            compile_src(src, args.output, args.file, start);
        },
        // Run
        (false, true) => {
            let src = read_to_string(&args.file).unwrap();
            run_src(src, debug);
        },
        (false, false) => {
            if args.file.extension() == Some("blsp".as_ref()) {
                let src = read_to_string(&args.file).unwrap();
                compile_src(src, args.output, args.file, start);
            } else if args.file.extension() == Some("bsm".as_ref()) {
                let src = read_to_string(&args.file).unwrap();
                run_src(src, debug);
            } else {
                panic!("No mode specified");
            }
        },
    }

}

fn compile_src(src: String, path: Option<PathBuf>, file: PathBuf, start: Instant) {
    let file_name = match path {
        Some(path) => path,
        None => Path::new(&file).to_path_buf(),
    }.file_stem().unwrap().to_str().unwrap().to_string();
    
    let tokens = tokenize(&cover_paren(src));
    let mut parser = Parser::new(tokens.clone());
    let result = parser.parse();

    match result {
        Ok(ast) => {
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
