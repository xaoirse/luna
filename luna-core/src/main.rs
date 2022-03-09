// Luna
// Developed by SAoirse
// xaoirse.github.com

use clap::Parser;
use colored::*;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::sync::{atomic::AtomicBool, Arc};

use model::*;

const BANNER: &str = r"
   __  __  ___  _____ 
  / / / / / / |/ / _ |  
 / /_/ /_/ /    / __ |    
/____|____/_/|_/_/ |_|  SA
";

fn main() {
    env_logger::init();
    debug!("Luna Begins.");
    run();
    debug!("Luna finished.");
}

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

    let opt = Opt::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.threads.unwrap_or_default())
        .build_global()
        .unwrap();

    if !opt.quiet {
        println!("{}", BANNER.cyan().bold());
    }
    let mut luna = Luna::parse();

    match opt.cli {
        Cli::Insert(insert) => {
            debug!("{:#?}", insert);

            let insert: Luna = (*insert).into();
            luna.append(insert);
            luna.dedup();
            luna.save();
        }

        Cli::Remove(find) => {
            debug!("{:#?}", find);
            let field = find.field;

            match find.filter.try_into() {
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
                Ok(filter) => {
                    let mut results = luna.find(field, &filter, find.verbose);
                    results.par_sort();
                    results.dedup();
                    results
                        .iter()
                        .take(filter.n)
                        .for_each(|r| println!("{}", r));
                }
                Err(err) => error!("Use fucking correct patterns: {}", err),
            }
        }

        Cli::Script(script) => {
            debug!("{:#?}", script);

            match script.parse() {
                Ok(script) => {
                    luna.dedup();
                    script.run(&mut luna, term);
                    info!("Scripts Executed.");
                }
                Err(err) => error!("Error in parsing file: {}", err),
            }
        }

        Cli::Import { file } => match Luna::from_file(&file) {
            Ok(file) => {
                luna.append(file);
                luna.dedup();
                luna.save();
            }
            Err(err) => error!("Can't import: {}", err),
        },

        Cli::Check(check) => {
            let input = &opt.input;

            match Luna::from_file(input) {
                Ok(mut luna) => {
                    luna.dedup();
                    println!("{} {}: {}", "[+]".green(), luna.stringify(1), input);
                    luna.save();
                }
                Err(err) => println!("{} Error in loading luna: {}", "[-]".red(), err),
            }

            if let Some(script_path) = check.script.as_ref() {
                let script = ScriptCli {
                    verbose: 0,
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
        Cli::Report => todo!(),
        Cli::Server(_) => todo!(),
    }
}
