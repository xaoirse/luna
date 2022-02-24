use colored::Colorize;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use structopt::StructOpt;

use super::script::ScriptCli;
use crate::model::url::Url;
use crate::model::*;

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
    script: Option<String>,
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

const BANNER: &str = r"
   __  __  ___  _____ 
  / / / / / / |/ / _ |  
 / /_/ /_/ /    / __ |    
/____|____/_/|_/_/ |_|  SA
";

pub fn run() {
    let term = Arc::new(AtomicBool::new(false));
    match signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)) {
        Ok(s) => info!("Luna has Graceful shutdown :) , SigId: {:#?}", s),
        Err(err) => {
            error!("Error in making signal-hook : {}", err);
            warn!("Luna will continue without Graceful shutdown");
        }
    }

    debug!("Running...");

    let opt = Opt::from_args();

    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.threads.unwrap_or_default())
        .build_global()
        .unwrap();

    if !opt.quiet {
        println!("{}", BANNER.cyan().bold());
    }
    let mut luna = Luna::from_args();

    match opt.cli {
        Cli::Insert(insert) => {
            debug!("{:#?}", insert);

            let insert: Luna = (*insert).into();
            luna.append(insert);
            luna.dedup(term);
            luna.save();
        }

        Cli::Remove(find) => {
            debug!("{:#?}", find);
            let field = find.field;

            match find.filter.clone().try_into() {
                Ok(filter) => {
                    if luna.remove(field, &filter) {
                        luna.save();
                    }
                }
                Err(err) => error!("Use fucking correct patterns: {}", err),
            }
        }

        Cli::Find(find) => {
            debug!("{:#?}", find);
            let field = find.field;
            match find.filter.try_into() {
                Ok(find) => {
                    let mut results = luna.find(field, &find);
                    results.par_sort();
                    results.dedup();
                    results.iter().take(find.n).for_each(|r| println!("{}", r));
                }
                Err(err) => error!("Use fucking correct patterns: {}", err),
            }
        }

        Cli::Script(script) => {
            debug!("{:#?}", script);

            match script.parse() {
                Ok(script) => {
                    luna.dedup(term.clone());
                    script.run(&mut luna, term);
                    info!("Scripts Executed.");
                }
                Err(err) => error!("Error in parsing file: {}", err),
            }
        }

        Cli::Import { file } => match Luna::from_file(&file) {
            Ok(file) => {
                luna.append(file);
                luna.dedup(term);
                luna.save();
            }
            Err(err) => error!("Can't import: {}", err),
        },

        Cli::Check(check) => {
            let input = &opt.input;

            match Luna::from_file(input) {
                Ok(mut luna) => {
                    luna.dedup(term);
                    println!("{} {}: {}", "[+]".green(), luna.stringify(1), input);
                    luna.save();
                }
                Err(err) => println!("{} Error in loading luna: {}", "[-]".red(), err),
            }

            if let Some(script_path) = check.script.as_ref() {
                let script = ScriptCli {
                    path: script_path.to_string(),
                    filter: Filter::default(),
                };
                match script.parse() {
                    Ok(_) => println!("{} Script: {}", "[+]".green(), script_path),
                    Err(err) => println!("{} {}", "[-]".red(), err),
                }
            } else {
                println!("[ ] No script file detected!")
            }
        }
        Cli::Stat(s) => println!("{}", luna.stringify(s.verbose + 2)),
        Cli::Test { n } => Luna::test_run(n),
        Cli::Report => todo!(),
        Cli::Server(_) => todo!(),
    }
}
