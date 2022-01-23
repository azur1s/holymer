use std::{fs::read_to_string, env::args, path::Path};

mod token;
mod util;
mod lexer;
mod parser;
mod compiler;

fn main() {
    let args = args().nth(1).expect("No input file");

    let src = util::cover_paren(read_to_string(&args).unwrap());
    let _file_name = Path::new(&args).file_stem().unwrap().to_str().unwrap();

    let tokens = lexer::lexer(&src);
    if tokens.is_err() {
        eprintln!("{}", tokens.unwrap_err());
        return;
    } else {
        // for t in tokens.as_ref().unwrap() {
        //     println!("{:?}", t);
        // }
        let ast = parser::parse(tokens.unwrap(), &args);
        if ast.is_err() {
            eprintln!("{}", ast.as_ref().unwrap_err());
            return;
        } else {
            let c = parser::translate_expr(ast.unwrap());
            match c {
                Ok(e) => {
                    println!("{:#?}", e);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
    }
}
