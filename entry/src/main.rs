use compiler::Compiler;
use lower::{model::converts, Lower};
use parser::{lex, parse, report};
use vm::exec::Executor;

fn main() {
    let path = std::env::args().nth(1).expect("No file path provided");
    let src = std::fs::read_to_string(path).expect("Failed to read file");

    let (tokens, lex_errors) = lex(src.to_string());
    let parse_errors = if let Some(tokens) = tokens {
        let (ast, parse_errors) = parse(tokens, src.len());

        if let Some(ast) = ast {
            let stripped = converts(ast);
            let mut lower = Lower::new();
            let lowered = lower.opt_stmts(stripped);
            let mut compiler = Compiler::new();
            let instrs = compiler.compile_program(lowered);
            // instrs.iter().for_each(|i| println!("{:?}", i));
            let mut executor = Executor::new(instrs);
            match executor.run() {
                Ok(_) => {}
                Err(e) => println!("Runtime error: {:?}", e),
            }
        }

        parse_errors
    } else {
        Vec::new()
    };

    if !lex_errors.is_empty() || !parse_errors.is_empty() {
        lex_errors
            .into_iter()
            .map(|e| e.map(|c| c.to_string()))
            .chain(parse_errors.into_iter().map(|e| e.map(|t| t.to_string())))
            .for_each(|e| report(e, &src));
    }
}
