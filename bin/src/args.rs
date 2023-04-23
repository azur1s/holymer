use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// The path to the file to be compiled.
    #[arg(required = true)]
    pub file: String,
}

pub fn get_args() -> Args {
    Args::parse()
}