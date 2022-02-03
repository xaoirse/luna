use clap::arg_enum;
use colored::Colorize;
use log::{debug, error, info, warn};
use structopt::StructOpt;

use super::script;
use crate::model::*;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short, long, global = true)]
    pub quiet: bool,
    #[structopt(short, long, default_value = "luna.json", global = true)]
    pub json: String,
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
    #[structopt(possible_values = &Fields::variants(), case_insensitive = true, help="Case Insensitive")]
    pub field: Fields,
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

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
    #[derive(Debug, Clone, Copy)]
    pub enum Fields {
        None,
        Keyword,
        Tech,
        Service,
        IP,
        Url,
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

impl From<&Fields> for &str {
    fn from(f: &Fields) -> Self {
        match f {
            Fields::Program => "program",
            Fields::Scope => "scope",
            Fields::Sub => "sub",
            Fields::Url => "url",
            Fields::IP => "ip",
            Fields::Keyword => "keyword",
            Fields::Service => "port",
            Fields::None => "",
            Fields::Tech => todo!(),
        }
    }
}

impl Fields {
    pub fn substitution(&self) -> String {
        let f: &str = self.into();
        format!("${{{}}}", f)
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
}

static BANNER: &str = r"
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.4.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    
";

pub fn run() {
    debug!("Running...");

    let opt = Opt::from_args();

    let json = &opt.json;
    if !opt.quiet {
        println!("{}", BANNER.blue());
    }

    let mut luna = match Luna::from_file(json) {
        Ok(luna) => {
            info!("Luna loaded successfully.");
            luna
        }
        Err(err) => {
            warn!("Can't load Luna from file!: {}", err);
            warn!("Empty Luna will be used!");
            Luna::default()
        }
    };

    match opt.cli {
        Cli::Insert(insert) => {
            debug!("{:#?}", insert);

            let insert: Luna = (*insert).into();
            luna.merge(insert);

            // TODO better mechanism for retry saving in errors
            if let Err(err) = luna.save(json) {
                error!("Error while saving: {}", err);
            } else {
                info!("Saved in \"{}\" successfully.", json);
            }
        }

        Cli::Find(find) => {
            debug!("{:#?}", find);

            let results = luna.find(&find);
            results.iter().for_each(|r| println!("{}", r));
        }

        Cli::Script(script) => {
            debug!("{:#?}", script);

            match script::parse(script.path) {
                Ok(script) => {
                    script.run(&luna).into_iter().for_each(|r| match r {
                        Ok(i) => luna.merge(i),
                        Err(err) => error!("{}", err),
                    });
                    info!("Scripts completed.");

                    if let Err(err) = luna.save(json) {
                        error!("Error while saving: {}", err);
                    } else {
                        info!("Saved in \"{}\" successfully.", json);
                    }
                }
                Err(err) => error!("Error in parsing file: {}", err),
            }
        }

        Cli::Check => todo!(),
        Cli::Test => todo!(),
        Cli::Report => todo!(),
        Cli::Server(_) => todo!(),
    }
}
