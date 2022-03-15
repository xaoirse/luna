use std::path::PathBuf;

use super::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "Luna",author, version, about, long_about = None)]
pub struct Opt {
    #[clap(short, long, global = true, help = "Quiet mode")]
    pub quiet: bool,
    #[clap(
        short,
        long,
        default_value = "luna.json",
        global = true,
        help = "Json file's path"
    )]
    pub input: String,
    #[clap(short, long, global = true, help = "Default output is input!")]
    pub output: Option<String>,
    #[clap(long, global = true, help = "Save without backup")]
    pub no_backup: bool,
    #[clap(short, long, global = true, help = "Number of threads")]
    pub threads: Option<usize>,
    #[clap(subcommand)]
    pub cli: Cli,
}

#[derive(Subcommand)]
pub enum Cli {
    #[clap(subcommand)]
    Insert(Box<Insert>),
    Remove(Box<Find>),
    Find(Box<Find>),
    Script(Box<ScriptCli>),
    Import {
        file: PathBuf,
    },
    Check(Check),
    Stat(LunaStat),
    Report,
    #[clap(subcommand)]
    Server(Server),
}

#[derive(Parser)]
pub struct Find {
    #[clap(arg_enum, ignore_case = true, help = "Case Insensitive")]
    pub field: Field,
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    #[clap(flatten)]
    pub filter: Filter,
}

#[derive(Debug, Parser)]
pub struct Check {
    #[clap(short, long)]
    pub script: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct LunaStat {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
}

#[derive(Debug, Parser)]
pub enum Insert {
    Program(InsertProgram),
    Asset(InsertAsset),
    Tag(InsertTag),
}

#[derive(Debug, Parser)]
pub struct InsertProgram {
    #[clap(flatten)]
    pub program: Program,
}

#[derive(Debug, Parser)]
pub struct InsertAsset {
    #[clap(short, long)]
    pub program: Option<Program>,
    #[clap(flatten)]
    pub asset: Asset,
}
#[derive(Debug, Parser)]
pub struct InsertTag {
    #[clap(short, long)]
    pub asset: AssetName,
    #[clap(flatten)]
    pub tag: Tag,
}

#[derive(Debug, Parser)]
pub enum Server {
    Start,
    Check,
    Report,
    Status,
}
