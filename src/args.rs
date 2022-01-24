use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "blspc")]
pub struct Args {
    /// Verbose mode (-v, -vv, -vvv, etc.). Max is 2 currently.
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Files to process.
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,
}