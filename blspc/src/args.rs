use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "blspc")]
pub struct Args {
    /// Verbose mode (-v, -vv, -vvv, etc.). Max is 2 currently.
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Compliation mode (-c).
    #[structopt(short, long)]
    pub compile: bool,

    /// Run mode (-r).
    #[structopt(short, long)]
    pub run: bool,

    /// Files to process.
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,

    /// Output file.
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,
}