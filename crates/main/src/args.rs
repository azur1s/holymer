use std::path::PathBuf;
use clap::{ Parser, Subcommand };

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Hades compiler.
#[derive(Parser, Debug)]
#[clap(
    version = VERSION,
    long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub options: Options,
}

#[derive(Subcommand, Debug)]
pub enum Options {
    #[clap(about = "Compile an input file.")]
    Compile {
        /// The input file to compile.
        #[clap(parse(from_os_str))]
        input: PathBuf,
        /// Print parsed AST and exit (for debugging).
        #[clap(short, long)]
        ast: bool,
    },
}