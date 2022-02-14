use colored::Colorize;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use structopt::StructOpt;

use super::script;
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
    pub json: String,
    #[structopt(long, global = true, help = "Save without backup")]
    pub no_backup: bool,
    #[structopt(subcommand)]
    pub cli: Cli,
}

#[derive(Debug, StructOpt)]
pub enum Cli {
    Insert(Box<Insert>),
    Find(Box<Filter>),
    Script(Script),
    #[structopt(help = "not implemented yet")]
    Check,
    #[structopt(help = "not implemented yet")]
    Test,
    #[structopt(help = "not implemented yet")]
    Report,
    #[structopt(help = "not implemented yet")]
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
    Tech(InsertTech),
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
pub struct InsertTech {
    #[structopt(flatten)]
    pub tech: Tech,
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
            if err.to_string() == "No such file or directory (os error 2)" {
                warn!("Can't load Luna from file! New filw will be generated.")
            } else {
                error!("Can't load Luna from file!: {}", err);
            }
            Luna::default()
        }
    };

    match opt.cli {
        Cli::Insert(insert) => {
            debug!("{:#?}", insert);

            let insert: Luna = (*insert).into();
            luna.append(insert);
            luna.merge();

            if let Err(err) = luna.save(json) {
                error!("Error while saving: {}", err);
            } else {
                info!("Saved in \"{}\" successfully.", json);
            }
        }

        Cli::Find(find) => {
            debug!("{:#?}", find);

            match (*find).try_into() {
                Ok(find) => {
                    let mut results = luna.find(&find);
                    results.par_sort();
                    results.dedup();
                    results.iter().take(find.n).for_each(|r| println!("{}", r));
                }
                Err(err) => error!("Use fucking right regex: {}", err),
            }
        }

        Cli::Script(script) => {
            debug!("{:#?}", script);

            match script::parse(script.path) {
                Ok(script) => {
                    script.run(&mut luna);
                    info!("Scripts Executed.");
                    luna.merge();

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
