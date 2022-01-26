use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "blvm")]
pub struct Args {
    /// Files to process.
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,
}