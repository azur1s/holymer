use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,

    #[structopt(short, long = "out", parse(from_os_str))]
    pub output: Option<PathBuf>,
}
