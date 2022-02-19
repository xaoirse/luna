use colored::Colorize;
use log::{debug, error, info, warn};
use rayon::prelude::*;
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
    Find(Box<Filter>),
    Script(ScriptCli),
    Import { file: String },
    Check(Check),
    Luna(LunaStat),
    Test,
    Report,
    Server(Server),
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
    Tech(InsertTech),
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

static BANNER: &str = r"
   __  __  ___  _____ 
  / / / / / / |/ / _ |    
 / /_/ /_/ /    / __ |    
/____|____/_/|_/_/ |_|  SA
";

pub fn run() {
    debug!("Running...");

    let opt = Opt::from_args();

    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.threads.unwrap_or_default())
        .build_global()
        .unwrap();

    let input = &opt.input;
    let output = opt.output.as_ref().unwrap_or(input);
    if !opt.quiet {
        println!("{}", BANNER.cyan().bold());
    }
    let mut luna = match Luna::from_file(input) {
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
            luna.dedup();

            if let Err(err) = luna.save(output) {
                error!("Error while saving: {}", err);
            } else {
                info!("Saved in \"{}\" successfully.", output);
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
                Err(err) => error!("Use fucking correct patterns: {}", err),
            }
        }

        Cli::Script(script) => {
            debug!("{:#?}", script);

            match script.parse() {
                Ok(script) => {
                    script.run(&mut luna);
                    info!("Scripts Executed.");
                    luna.dedup();

                    if let Err(err) = luna.save(output) {
                        error!("Error while saving: {}", err);
                    } else {
                        info!("Saved in \"{}\" successfully.", output);
                    }
                }
                Err(err) => error!("Error in parsing file: {}", err),
            }
        }

        Cli::Import { file } => match Luna::from_file(&file) {
            Ok(file) => {
                luna.append(file);
                luna.dedup();
                if let Err(err) = luna.save(output) {
                    error!("Error while saving in \"{}\": {}", output, err);
                } else {
                    info!("Saved in \"{}\" successfully.", output);
                }
            }
            Err(err) => error!("Can't import: {}", err),
        },

        Cli::Check(check) => {
            match Luna::from_file(input) {
                Ok(luna) => println!("{} {}: {}", "[+]".green(), luna.stringify(1), input),
                Err(_) => println!("{} ", "[-]".red()),
            }

            if let Some(script) = check.script.as_ref() {
                let script = ScriptCli {
                    path: script.to_string(),
                    updated_at: None,
                    started_at: None,
                };
                match script.parse() {
                    Ok(_) => println!("{} Script: {}", "[+]".green(), script.path),
                    Err(err) => println!("{} {}", "[-]".red(), err),
                }
            } else {
                println!("[ ] No script file detected!")
            }
        }
        Cli::Luna(s) => println!("{}", luna.stringify(s.verbose)),
        Cli::Test => todo!(),
        Cli::Report => todo!(),
        Cli::Server(_) => todo!(),
    }
}
