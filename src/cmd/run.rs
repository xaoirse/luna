use clap::arg_enum;
use structopt::StructOpt;

use crate::model::*;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    pub cli: Cli,
}

#[derive(Debug, StructOpt)]
pub enum Cli {
    Insert(Box<Insert>),
    Find(Box<Filter>),
    Script(Script),
    Check,
    Test,
    Report,
    Server(Server),
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

#[derive(Debug, StructOpt, Default)]
pub struct Filter {
    #[structopt(possible_values = &Fields::variants(), case_insensitive = true)]
    pub field: Fields,

    #[structopt(long, short)]
    pub program: Option<String>,
    #[structopt(long)]
    pub program_platform: Option<String>,
    #[structopt(long)]
    pub program_type: Option<String>,
    #[structopt(long)]
    pub program_bounty: Option<String>,
    #[structopt(long)]
    pub program_state: Option<String>,

    #[structopt(long, short)]
    pub scope: Option<String>,
    #[structopt(long)]
    pub scope_type: Option<String>,
    #[structopt(long)]
    pub scope_bounty: Option<String>,

    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(long)]
    pub ip: Option<String>,

    #[structopt(long)]
    pub port: Option<String>,
    #[structopt(long)]
    pub service_name: Option<String>,

    #[structopt(long, short = "u")]
    pub url: Option<String>,
    #[structopt(long)]
    pub title: Option<String>,
    #[structopt(long, short = "c")]
    pub status_code: Option<String>,
    #[structopt(long)]
    pub content_type: Option<String>,
    #[structopt(long)]
    pub content_length: Option<String>,

    #[structopt(long)]
    pub tech: Option<String>,
    #[structopt(long)]
    pub tech_version: Option<String>,

    #[structopt(long, short = "m")]
    pub minutes_before: Option<i32>,
    #[structopt(long, short = "d")]
    pub days_before: Option<i32>,
}

arg_enum! {
    #[derive(Debug)]
    pub enum Fields {
        URL,
        Sub,
        Scope,
        Program,
    }
}
impl Default for Fields {
    fn default() -> Self {
        Self::Scope
    }
}

#[derive(Debug, StructOpt)]
pub struct Script {
    pub path: String,
}

#[derive(Debug, StructOpt)]
pub enum Server {
    Start,
    Check,
    Report,
    Status,
    //TODO
}

pub async fn run() {
    let opt = Opt::from_args();
    let path = "luna.json";

    match opt.cli {
        Cli::Insert(insert) => {
            let insert: Luna = match *insert {
                Insert::Program(i) => i.into(),
                Insert::Scope(i) => i.into(),
                Insert::Scopes(i) => i.into(),
                Insert::Sub(i) => i.into(),
                Insert::Subs(i) => i.into(),
                Insert::Url(i) => i.into(),
                Insert::Urls(i) => i.into(),
                Insert::Host(i) => i.into(),
                Insert::Hosts(i) => i.into(),
            };
            let mut file = Luna::from_file(path).unwrap_or_default();
            file.merge(insert);
            file.save(path).unwrap();
        }
        Cli::Find(find) => {
            let luna = Luna::from_file(path).unwrap_or_default();
            let results = luna.find(&find);
            results.iter().for_each(|r| println!("{}", r))
        }
        _ => (),
    }
}

// TODO
// luna find sub --url index.html --status-code 200
