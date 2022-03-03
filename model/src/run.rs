use super::*;
use ::url as urlib;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
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

#[derive(Debug, Subcommand)]
pub enum Cli {
    #[clap(subcommand)]
    Insert(Box<Insert>),
    Remove(Box<FindCli>),
    Find(Box<FindCli>),
    Script(Box<ScriptCli>),
    Import {
        file: String,
    },
    Check(Check),
    Stat(LunaStat),
    Report,
    #[clap(subcommand)]
    Server(Server),
}

#[derive(Debug, Parser)]
pub struct FindCli {
    #[clap(arg_enum, ignore_case = true, help = "Case Insensitive")]
    pub field: Fields,

    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[clap(flatten)]
    pub filter: Filter,
}

#[derive(Debug, Parser)]
pub struct Check {
    #[clap(short, long)]
    pub script: Option<String>,
}

#[derive(Debug, Parser)]
pub struct LunaStat {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
}

#[derive(Debug, Parser)]
pub enum Insert {
    Program(InsertProgram),
    Scope(InsertScope),
    Scopes(InsertScopes),
    Sub(InsertSub),
    Subs(InsertSubs),
    Url(InsertUrl),
    Urls(InsertUrls),
    Host(InsertHost),
    Hosts(InsertHosts),
    Tag(InsertTag),
    Service(InsertService),
}

#[derive(Debug, Parser)]
pub struct InsertProgram {
    #[clap(flatten)]
    pub program: Program,
}

#[derive(Debug, Parser)]
pub struct InsertScope {
    #[clap(flatten)]
    pub scope: Scope,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InsertScopes {
    pub scopes: Vec<Scope>,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InsertSub {
    #[clap(flatten)]
    pub sub: Sub,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InsertSubs {
    pub subs: Vec<Sub>,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InsertUrl {
    #[clap(flatten)]
    pub url: Url,
    #[clap(long)]
    pub sub: Option<String>,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}
#[derive(Debug, Parser)]
pub struct InsertUrls {
    pub urls: Vec<Url>,
    #[clap(long)]
    pub sub: Option<String>,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InsertHost {
    #[clap(flatten)]
    pub host: Host,
    #[clap(long)]
    pub sub: Option<String>,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}
#[derive(Debug, Parser)]
pub struct InsertHosts {
    pub hosts: Vec<Host>,
    #[clap(long)]
    pub sub: Option<String>,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InsertTag {
    #[clap(flatten)]
    pub tag: Tag,
    #[clap(long)]
    pub url: urlib::Url,
    #[clap(long)]
    pub sub: Option<String>,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InsertService {
    #[clap(flatten)]
    pub service: Service,
    #[clap(long)]
    pub host: std::net::IpAddr,
    #[clap(long)]
    pub sub: Option<String>,
    #[clap(short, long)]
    pub scope: Option<String>,
    #[clap(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, Parser)]
pub enum Server {
    Start,
    Check,
    Report,
    Status,
}
