use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "blspc")]
pub struct Opts {
    #[structopt(subcommand)]
    pub commands: Option<Args>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "args")]
pub enum Args {
    #[structopt(name = "compile")]
    Compile (CompileOpts),
    #[structopt(name = "run")]
    Run (RunOpts),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "compile", about = "Compile Options")]
pub struct CompileOpts {
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,
    #[structopt(name = "OUTPUT", parse(from_os_str))]
    pub output: Option<PathBuf>,
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(short = "b", long)]
    pub with_comment: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "run", about = "Run Options")]
pub struct RunOpts {
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,
    #[structopt(short, long)]
    pub debug: bool,
}