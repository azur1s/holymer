use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// The path to the file to be compiled.
    #[arg(required = true)]
    pub file: String,
    /// Only run the type checker.
    #[arg(short = 'c', long = "check")]
    pub typecheck: bool,
}

pub fn get_args() -> Args {
    Args::parse()
}