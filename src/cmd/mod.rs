use crate::model::{self, *};
use crate::tools;
use crate::tools::extractor::Extractor;
use futures::future::join_all;
use futures::join;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Insert {
    Program(Program),
    Scope(Scope),
    Sub(Sub),
    Host(Host),
    URL(URL),
    Service(Service),
    Tech(Tech),
    // Job(Job),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "The Moon Rider has arrived.\nmongodb")]
pub struct Opt {
    // #[structopt(short, long, help = "mysql://example.com/test")]
    // pub db_url: Option<String>,
    #[structopt(subcommand)]
    pub sub: Subcommand,
}
#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Insert(Insert),
    Find {
        ty: String,

        #[structopt(help = "Filter objects\nYou can see Data structues in source.")]
        filter: Option<String>,

        #[structopt(short, help = "The number of results")]
        n: Option<String>,

        #[structopt(short, long, help = "Query for sort")]
        sort: Option<String>,

        #[structopt(short, long, help = "Field to show")]
        field: Option<String>,
    },
    Script {
        #[structopt(short, long)]
        script_name: String,

        #[structopt(long)]
        all_scopes: bool,

        #[structopt(long)]
        all_subs: bool,

        #[structopt(long)]
        all_hosts: bool,

        #[structopt(long)]
        all_urls: bool,

        entries: Vec<String>,
    },
}

pub async fn from_args() {
    let opt = Opt::from_args();
    // Match subcommands: insert, find
    match opt.sub {
        Subcommand::Insert(insert) => match insert {
            Insert::Program(doc) => {
                model::update(doc).await;
            }
            Insert::Scope(doc) => {
                model::update(doc).await;
            }
            Insert::Sub(doc) => {
                model::update(doc).await;
            }
            Insert::Host(doc) => {
                model::update(doc).await;
            }
            Insert::URL(doc) => {
                model::update(doc).await;
            }
            Insert::Service(doc) => {
                model::update(doc).await;
            }
            Insert::Tech(doc) => {
                model::update(doc).await;
            }
        },
        Subcommand::Find {
            ty,
            filter,
            n,
            sort,
            field,
        } => match ty.as_str() {
            "program" => {
                model::find_as_string::<Program>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "scope" => {
                model::find_as_string::<Scope>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "sub" => {
                model::find_as_string::<Sub>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "host" => {
                model::find_as_string::<Host>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "url" => {
                model::find_as_string::<URL>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "service" => {
                model::find_as_string::<Service>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "tech" => {
                model::find_as_string::<Tech>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "job" => {
                format!("I'm not sure about implementing this for now").warn();
            }
            typ => format!("Strut '{}' not found!", typ).error(),
        },

        Subcommand::Script {
            script_name,
            all_scopes,
            all_subs,
            all_hosts,
            all_urls,
            mut entries,
        } => {
            if all_scopes {
                entries.append(
                    &mut model::find_as_vec::<Scope>(None, None, None)
                        .await
                        .into_par_iter()
                        .map(|t| t.asset)
                        .collect(),
                );
            }
            if all_subs {
                entries.append(
                    &mut model::find_as_vec::<Sub>(None, None, None)
                        .await
                        .into_par_iter()
                        .map(|t| t.asset)
                        .collect(),
                )
            };
            if all_hosts {
                entries.append(
                    &mut model::find_as_vec::<Host>(None, None, None)
                        .await
                        .into_par_iter()
                        .map(|t| t.ip)
                        .collect(),
                )
            };
            if all_urls {
                // TODO use url = "2.2.2";
                entries.append(
                    &mut model::find_as_vec::<URL>(None, None, None)
                        .await
                        .into_par_iter()
                        .map(|t| t.url)
                        .collect(),
                )
            };

            let script_name = Arc::new(script_name);
            let wl = Arc::new(Mutex::new(Vec::new()));
            // let now = std::time::Instant::now();

            let e = entries.iter().map(|entry| {
                let mut key_vals = HashMap::new();
                key_vals.insert("$$", vec![entry.clone()]);

                // Run script
                let output = Arc::new(tools::run_script(key_vals, script_name.clone()));

                // Get future for running extractors
                let subs =
                    tokio::task::spawn(join_all(output.clone().extract_fut(|mut t: Sub| {
                        {
                            wl.lock().unwrap().append(&mut t.wordlister());
                        }
                        t.scope = entry.clone();
                        crate::model::update(t)
                    })));

                let hosts =
                    tokio::task::spawn(join_all(output.clone().extract_fut(|mut t: Host| {
                        {
                            wl.lock().unwrap().append(&mut t.wordlister());
                        }
                        t.sub = entry.clone();
                        crate::model::update(t)
                    })));

                let urls = tokio::task::spawn(join_all(output.extract_fut(|mut t: URL| {
                    {
                        wl.lock().unwrap().append(&mut t.wordlister());
                    }
                    t.sub = entry.clone();
                    crate::model::update(t)
                })));

                // Gather all futures for Run all extractors in parallel
                tokio::spawn(async { join!(subs, hosts, urls) })
            });

            // Run all extractors for all entries in parallel
            join_all(e).await;

            // println!("{}", now.elapsed().as_millis());

            crate::tools::file::save("wl.txt", wl.lock().unwrap().to_vec());
        }
    }
}
