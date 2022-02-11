use std::{fs::{ read_to_string, File }, io::Write};
use clap::Parser;

/// Arguments handler.
pub mod args;
use args::{ Args, Options };

/// A front-end for the compiler.
/// Contains parser and tokenizer.
/// TODO: Semantic analysis and Type checking.
pub mod front;
use front::parser::parse;

/// A middle-end for the compiler.
/// Contains high intermediate representation (HIR).
pub mod middle;
use crate::middle::hir::to_hirs;

fn main() {
    let args = Args::parse();
    match args.options {
        Options::Compile { input, ast } => {
            let code = read_to_string(&input).unwrap();
            let tree = parse(&code);
            match ast {
                true => for node in tree { println!("{:#?}", node) },
                false => {
                    // Check if the tree is valid
                    let mut checked_tree = Vec::new();
                    for node in tree {
                        match node {
                            Ok(node) => checked_tree.push(node.0),
                            Err(err) => println!("{:?}", err),
                        }
                    };

                    // Convert the tree to HIR
                    let hir = to_hirs(&checked_tree);
                    println!("{:#?}", hir);
                },
            }
        },
    }
}