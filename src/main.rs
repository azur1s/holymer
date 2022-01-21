use std::{fs::read_to_string, env::args, path::Path};

mod token;
mod util;
mod lexer;
mod parser;

fn main() {
    let args = args().nth(1).expect("No input file");

    let src = util::cover_paren(read_to_string(&args).unwrap());
    let _file_name = Path::new(&args).file_stem().unwrap().to_str().unwrap();

    let tokens = lexer::lexer(&src);
    if tokens.is_err() {
        eprintln!("{}", tokens.as_ref().unwrap_err());
    }

    let ast = parser::parse(tokens.unwrap());
    if ast.is_err() {
        eprintln!("{:?}", ast.as_ref().unwrap_err());
    } else {
        // Everything is in a List(..) so we need to get it out and make it into
        // a vector of Expr instead, so we can compile it.
        let a = util::unwrap_list_nest(ast.unwrap());
        for e in a.iter() {
            println!("{}", e);
        }
        // TODO: compile to something else..
    }
}
