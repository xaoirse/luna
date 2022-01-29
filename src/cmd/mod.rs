pub mod run;
/*
use crate::model::*;
use structopt::StructOpt;

use crate::alert::Alert;
// use crate::database::mongo;

mod run;
pub mod run_sql;

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
    Check,
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
        Subcommand::Check => {
            run::check().await;
        }
        Subcommand::Insert(insert) => match insert {
            Insert::Program(doc) => {
                mongo::update(doc).await;
            }
            Insert::Scope(doc) => {
                mongo::update(doc).await;
            }
            Insert::Sub(doc) => {
                mongo::update(doc).await;
            }
            Insert::Host(doc) => {
                mongo::update(doc).await;
            }
            Insert::URL(doc) => {
                mongo::update(doc).await;
            }
            Insert::Service(doc) => {
                mongo::update(doc).await;
            }
            Insert::Tech(doc) => {
                mongo::update(doc).await;
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
                mongo::find_as_string::<Program>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "scope" => {
                mongo::find_as_string::<Scope>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "sub" => {
                mongo::find_as_string::<Sub>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "host" => {
                mongo::find_as_string::<Host>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "url" => {
                mongo::find_as_string::<URL>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "service" => {
                mongo::find_as_string::<Service>(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "tech" => {
                mongo::find_as_string::<Tech>(filter, n, sort, field)
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
                    &mut mongo::find_as_vec::<Scope>(None, None, None)
                        .await
                        .into_iter()
                        .map(|t| t.asset)
                        .collect(),
                );
            }
            if all_subs {
                entries.append(
                    &mut mongo::find_as_vec::<Sub>(None, None, None)
                        .await
                        .into_iter()
                        .map(|t| t.asset)
                        .collect(),
                )
            };
            if all_hosts {
                entries.append(
                    &mut mongo::find_as_vec::<Host>(None, None, None)
                        .await
                        .into_iter()
                        .map(|t| t.ip)
                        .collect(),
                )
            };
            if all_urls {
                // TODO use url = "2.2.2";
                entries.append(
                    &mut mongo::find_as_vec::<URL>(None, None, None)
                        .await
                        .into_iter()
                        .map(|t| t.url)
                        .collect(),
                )
            };

            run::run(entries, script_name).await;
        }
    }
}

*/
