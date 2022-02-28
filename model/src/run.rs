use structopt::StructOpt;

use super::*;

#[derive(Debug, StructOpt)]
#[structopt(author = "SA", about = "The moon rider has arived.")]
pub struct Opt {
    #[structopt(short, long, global = true, help = "Quiet mode")]
    pub quiet: bool,
    #[structopt(
        short,
        long,
        default_value = "luna.json",
        global = true,
        help = "Json file's path"
    )]
    pub input: String,
    #[structopt(short, long, global = true, help = "Default output is input!")]
    pub output: Option<String>,
    #[structopt(long, global = true, help = "Save without backup")]
    pub no_backup: bool,
    #[structopt(short, long, global = true, help = "Number of threads")]
    pub threads: Option<usize>,
    #[structopt(subcommand)]
    pub cli: Cli,
}

#[derive(Debug, StructOpt)]
pub enum Cli {
    Insert(Box<Insert>),
    Remove(Box<FindCli>),
    Find(Box<FindCli>),
    Script(Box<ScriptCli>),
    Import { file: String },
    Check(Check),
    Stat(LunaStat),
    Test { n: i32 },
    Report,
    Server(Server),
}

#[derive(Debug, StructOpt)]
pub struct FindCli {
    #[structopt(possible_values = &Fields::variants(), case_insensitive = true, help="Case Insensitive")]
    pub field: Fields,

    #[structopt(flatten)]
    pub filter: Filter,
}

#[derive(Debug, StructOpt)]
pub struct Check {
    #[structopt(short, long)]
    pub script: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct LunaStat {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}

#[derive(Debug, StructOpt)]
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

#[derive(Debug, StructOpt)]
pub struct InsertProgram {
    #[structopt(flatten)]
    pub program: Program,
}

#[derive(Debug, StructOpt)]
pub struct InsertScope {
    #[structopt(flatten)]
    pub scope: Scope,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct InsertScopes {
    pub scopes: Vec<Scope>,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct InsertSub {
    #[structopt(flatten)]
    pub sub: Sub,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct InsertSubs {
    pub subs: Vec<Sub>,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct InsertUrl {
    #[structopt(flatten)]
    pub url: Url,
    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}
#[derive(Debug, StructOpt)]
pub struct InsertUrls {
    pub urls: Vec<Url>,
    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct InsertHost {
    #[structopt(flatten)]
    pub host: Host,
    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}
#[derive(Debug, StructOpt)]
pub struct InsertHosts {
    pub hosts: Vec<Host>,
    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct InsertTag {
    #[structopt(flatten)]
    pub tag: Tag,
    #[structopt(long)]
    pub url: String,
    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct InsertService {
    #[structopt(flatten)]
    pub service: Service,
    #[structopt(long)]
    pub host: String,
    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(short, long)]
    pub scope: Option<String>,
    #[structopt(short, long)]
    pub program: Option<String>,
}

#[derive(Debug, StructOpt)]
pub enum Server {
    Start,
    Check,
    Report,
    Status,
}
