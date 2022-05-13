use std::{fs::File, io::Write};

use syntax::{lex::lex, parse::parse};
use codegen::Codegen;

pub mod util;

fn main() {
    let path = std::env::args().nth(1).expect("No file specified");
    let input = std::fs::read_to_string(path).expect("Failed to read file");

    let time = std::time::Instant::now();

    //
    // Lex
    //
    let (tokens, lex_errs) = lex(input.to_string());

    if !lex_errs.is_empty() {
        println!("Lex error(s): {:#?}", lex_errs);
        return;
    }

    //
    // Parse
    //
    let (ast, parse_errs) = parse(tokens.unwrap(), input.chars().count());

    if !parse_errs.is_empty() || ast.is_none() {
        println!("Parse error(s): {:#?}", parse_errs);
        return;
    }

    println!("{:#?}", ast.as_ref().unwrap());
    info!("Parsed in {}ms", time.elapsed().as_millis());

    //
    // Codegen
    //
    let mut codegen = Codegen::new();
    codegen.gen(ast.unwrap());
    codegen.finalize();

    let mut file = File::create("out.ts").unwrap();
    file.write_all(&codegen.finalized).unwrap();

    info!("Generated {} bytes in {} ms", codegen.finalized.len(), time.elapsed().as_millis());
}
