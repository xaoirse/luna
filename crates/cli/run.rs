use clap::{Parser, Subcommand};
use colored::*;
use dnsgen::dnsgen;
use log::{debug, error, info, warn};
use model::filter::Regex;
use model::*;
use std::{
    path::PathBuf,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};

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
    pub input: PathBuf,
    #[clap(short, long, global = true, help = "Default output is input!")]
    pub output: Option<PathBuf>,
    #[clap(long, global = true, help = "Save without backup!")]
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
    Dnsgen(Dnsgen),
    Report(Report),
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
pub struct Dnsgen {
    pub sub: String,
    #[clap(short, long)]
    pub wl: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct Report {
    #[clap(short, long, default_value = ".")]
    pub path: PathBuf,
    #[clap(short, default_value = "md")]
    pub format: String,
}

#[derive(Debug, Parser)]
pub enum Server {
    Start { ip: String, port: u16 },
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
        Ok(s) => info!("Luna has Graceful shutdown :), SigId: {:?}", s),
        Err(err) => {
            error!("Error in making signal-hook: {}", err);
            warn!("Luna will continue without Graceful shutdown!");
        }
    }

    debug!("Running...");

    let opt = Opt::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.threads.unwrap_or_default())
        .build_global()
        .unwrap();

    if !opt.quiet {
        println!("{}", BANNER.cyan().bold());
    }
    let mut luna = Luna::parse(&opt.input);
    let output = opt.output.as_ref().unwrap_or(&opt.input);

    match opt.cli {
        Cli::Insert(insert) => {
            let res = match *insert {
                Insert::Program(p) => luna.insert_program(p.program),
                Insert::Asset(a) => luna.insert_asset(a.asset, a.program),
                Insert::Tag(t) => luna.insert_tag(t.tag, &t.asset),
            };

            match res {
                Ok(_) => luna.save(output, !opt.no_backup),
                Err(err) => warn!("{err}"),
            }
        }

        Cli::Remove(find) => {
            luna.remove(find.field, &find.filter);
            luna.save(output, !opt.no_backup);
        }

        Cli::Find(find) => {
            luna.find(find.field, &find.filter, find.verbose)
                .iter()
                .for_each(|r| println!("{}", r));
        }

        Cli::Script(script) => match script.parse() {
            Ok(script) => {
                script.run(&mut luna, output, opt.no_backup, term);
                info!("Scripts Executed.");
            }
            Err(err) => error!("Error in parsing file: {}", err),
        },

        Cli::Import { file } => match Luna::from_file(&file) {
            Ok(file) => {
                luna.merge(file);
                luna.save(output, !opt.no_backup)
            }
            Err(err) => error!("Can't import: {}", err),
        },

        Cli::Check(check) => {
            let input = &opt.input;

            match Luna::from_file(input) {
                Ok(luna) => {
                    println!(
                        "{} {}: {}",
                        "[+]".green(),
                        luna.stringify(1),
                        input.display()
                    );
                    luna.save(output, !opt.no_backup)
                }
                Err(err) => println!("{} Error in loading luna: {}", "[-]".red(), err),
            }

            if let Some(script_path) = check.script.as_ref() {
                let script = ScriptCli {
                    verbose: 0,
                    path: script_path.to_path_buf(),
                    filter: Filter::default(),
                };
                match script.parse() {
                    Ok(_) => println!(
                        "{} Script: {}",
                        "[+]".green(),
                        script_path.to_string_lossy()
                    ),
                    Err(err) => println!("{} {}", "[-]".red(), err),
                }
            } else {
                println!("[ ] No script file detected!")
            }
        }
        Cli::Stat(s) => println!("{}", luna.stringify(s.verbose + 2)),
        Cli::Dnsgen(dg) => {
            if let Ok(sub) = Regex::from_str(&dg.sub) {
                let wl = if let Some(path) = dg.wl {
                    std::fs::read_to_string(path)
                        .unwrap()
                        .split_ascii_whitespace()
                        .map(String::from)
                        .collect()
                } else {
                    vec!["dev".to_string(), "test".to_string()]
                };

                let filter = Filter {
                    asset: sub,
                    ..Default::default()
                };
                dnsgen(luna.find(Field::Sub, &filter, 0), wl)
                    .into_iter()
                    .for_each(|s| println!("{s}"))
            }
        }
        Cli::Report(_) => todo!(),
        Cli::Server(_) => todo!(),
    }
}
