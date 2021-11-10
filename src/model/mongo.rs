use mongodb::{
    bson::{doc, DateTime, Document},
    options::ClientOptions,
    Client, Database,
};
use rayon::prelude::*;
use std::collections::HashMap;

use clap::arg_enum;
// This trait is required to use `try_next()` on the cursor
use futures::stream::TryStreamExt;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::{
    model::Alert,
    tools::{self, extractor::Extractor},
};

arg_enum! {
    #[derive(Debug, Serialize, Deserialize, StructOpt,Clone,PartialEq, Eq)]
    pub enum ProgramPlatform {
        HackerOne,
        BugCrowd,
        SelfManaged,
        Anonymous,
    }
}
arg_enum! {
    #[derive(Debug, Serialize, Deserialize, StructOpt,Clone,PartialEq, Eq)]
    pub enum ProgramType {
        Public,
        Private,
    }
}
arg_enum! {
    #[derive(Debug, Serialize, Deserialize, StructOpt,Clone,PartialEq, Eq)]
    pub enum ProgramState {
        Open,
        Closed,
    }
}
#[derive(Debug, Serialize, Deserialize, StructOpt, orm::mongorm, Clone, PartialEq, Eq)]
pub struct Program {
    #[structopt(short, long, possible_values = &ProgramPlatform::variants(),case_insensitive = true)]
    pub platform: Option<ProgramPlatform>,

    #[structopt(short, long)]
    pub name: String, // id

    #[structopt(long)]
    pub handle: Option<String>,

    #[structopt(short, long,possible_values = &ProgramType::variants(),case_insensitive = true)]
    pub ty: Option<ProgramType>,

    #[structopt(short, long)]
    pub url: Option<String>,

    #[structopt(short, long)]
    pub icon: Option<String>,

    #[structopt(short, long)]
    pub bounty: Option<String>,

    #[structopt(long,possible_values = &ProgramState::variants(),case_insensitive = true)]
    pub state: Option<ProgramState>,

    #[rel("Scope")]
    #[structopt(short, long)]
    pub scopes: Vec<String>,

    // #[structopt(skip)]
    // started_at: Option<DateTime>,
    #[structopt(skip)]
    pub update: Option<DateTime>,
}
arg_enum! {
    #[derive(Debug, Serialize, Deserialize, StructOpt,Clone,PartialEq, Eq)]
    pub enum ScopeType {
        SingleDomain,
        WildcardDomain,
        IOS,
        Android,
        Windows,
        Mac,
        Linux,
        SourceCode,
        CIDR,
    }
}
arg_enum! {
    #[derive(Debug, Serialize, Deserialize, StructOpt,Clone,PartialEq, Eq)]
    pub enum ScopeSeverity {
        Critical,
        High,
        Medium,
        Low,
    }
}
#[derive(Debug, Serialize, Deserialize, StructOpt, orm::mongorm, Clone, PartialEq, Eq)]
pub struct Scope {
    #[structopt(short, long)]
    pub asset: String, // id

    #[structopt(short, long,possible_values = &ScopeType::variants(),case_insensitive = true)]
    pub ty: Option<ScopeType>,

    #[structopt(short, long)]
    pub eligible_bounty: Option<bool>,

    #[structopt(long,possible_values = &ScopeSeverity::variants(),case_insensitive = true)]
    pub severity: Option<ScopeSeverity>,

    #[structopt(short, long)]
    pub program: String,

    #[rel("Sub")]
    #[structopt(short, long)]
    pub subs: Vec<String>,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}
arg_enum! {
    #[derive(Clone,Debug, Serialize, Deserialize, StructOpt,PartialEq, Eq)]
    pub enum SubType {
        IP,
        Domain,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, StructOpt, orm::mongorm, PartialEq, Eq)]
pub struct Sub {
    #[structopt(short, long)]
    pub asset: String, // id

    #[structopt(short, long)]
    pub scope: String,

    #[structopt(short, long,possible_values = &SubType::variants(),case_insensitive = true)]
    pub ty: Option<SubType>,

    #[rel("Host")]
    #[structopt(long)]
    pub host: Option<String>,

    #[rel("URL")]
    #[structopt(short, long)]
    pub urls: Vec<String>,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}
#[derive(Debug, Serialize, Deserialize, StructOpt, orm::mongorm, Clone, PartialEq, Eq)]
pub struct Service {
    #[structopt(long)]
    pub port: String,

    #[structopt(short, long)]
    pub protocol: Option<String>,

    #[structopt(short, long)]
    pub banner: Option<String>,
}
#[derive(
    Debug, Serialize, Deserialize, StructOpt, orm::mongorm, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Host {
    #[structopt(short, long)]
    pub ip: String,

    #[rel("Service")]
    #[structopt(long)]
    pub services: Vec<String>,

    #[structopt(short, long)]
    pub sub: String,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}
#[derive(Debug, Serialize, Deserialize, StructOpt, orm::mongorm, Clone, PartialEq, Eq)]
pub struct Tech {
    #[structopt(short, long)]
    pub name: String,

    #[structopt(short, long)]
    pub version: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, StructOpt, orm::mongorm, Clone, PartialEq, Eq)]
pub struct URL {
    #[structopt(short, long)]
    pub url: String,

    #[structopt(short, long)]
    pub sub: String,
    #[structopt(long)]
    pub title: Option<String>,

    #[structopt(long)]
    pub status_code: Option<String>,

    #[structopt(short, long)]
    pub content_type: Option<String>,

    #[rel("Tech")]
    #[structopt(short, long)]
    pub techs: Vec<String>,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}

arg_enum! {
    #[derive(Debug, Serialize, Deserialize, StructOpt, Clone)]
    pub enum JobState {
        New,
        DataGathering,
        Processing,
        GeneratinOut,
        NeedsNotif,
        Archived,
    }
}

#[derive(Debug, Serialize, Deserialize, StructOpt, orm::mongorm, Clone)]
pub struct Job {
    // TODO write help for all fields
    #[structopt(short, long, help = "Task's name\nNot implemented yet!")]
    task: String,

    #[structopt(short, long)]
    extra_param: Option<String>,

    #[structopt(short, long)]
    input_files: Vec<String>,
    #[structopt(short, long)]
    output_files: Vec<String>,

    #[structopt(short, long)]
    program: Option<String>,

    #[structopt(short, long)]
    scope: String,

    #[structopt(long)]
    host: String,

    #[structopt(short, long)]
    url: String,

    #[structopt(long)]
    tech: Vec<String>,

    #[structopt(long,possible_values = &JobState::variants(),case_insensitive = true)]
    state: Option<JobState>,

    #[structopt(skip)]
    update: Option<DateTime>,
}

#[derive(Debug, StructOpt)]
pub enum Insert {
    Program(Program),
    Scope(Scope),
    Sub(Sub),
    Host(Host),
    URL(URL),
    Service(Service),
    Tech(Tech),
    Job(Job),
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

        #[structopt(
            help = "mongo query for filter objects\nYou can see Data structues in source."
        )]
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

pub async fn get_db() -> Database {
    // Get db_url
    let url = super::get_db_url().await;

    // Parse a connection String, into an options
    let mut client_options = ClientOptions::parse(url).await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("Luna app".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // Return a handle to a database.
    if cfg!(test) {
        client.database("test")
    } else {
        client.database("luna")
    }
}

pub async fn action_from_args(opt: Opt) {
    // Match subcommands: insert, find
    match opt.sub {
        Subcommand::Insert(insert) => match insert {
            Insert::Program(doc) => {
                doc.update().await;
            }
            Insert::Scope(doc) => {
                doc.update().await;
            }
            Insert::Sub(doc) => {
                doc.update().await;
            }
            Insert::Host(doc) => {
                doc.update().await;
            }
            Insert::URL(doc) => {
                doc.update().await;
            }
            Insert::Service(doc) => {
                doc.update().await;
            }
            Insert::Tech(doc) => {
                doc.update().await;
            }
            Insert::Job(doc) => {
                doc.update().await;
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
                Program::find_fields(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "scope" => {
                Scope::find_fields(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "sub" => {
                Sub::find_fields(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "host" => {
                Host::find_fields(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "url" => {
                URL::find_fields(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "service" => {
                Service::find_fields(filter, n, sort, field)
                    .await
                    .iter()
                    .for_each(|f| println!("{}", f));
            }
            "tech" => {
                Tech::find_fields(filter, n, sort, field)
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
                    &mut Scope::find(None, None, None)
                        .await
                        .into_par_iter()
                        .map(|t| t.asset)
                        .collect(),
                );
            }
            if all_subs {
                entries.append(
                    &mut Sub::find(None, None, None)
                        .await
                        .into_iter()
                        .map(|t| t.asset)
                        .collect(),
                )
            };
            if all_hosts {
                entries.append(
                    &mut Host::find(None, None, None)
                        .await
                        .into_par_iter()
                        .map(|t| t.ip)
                        .collect(),
                )
            };
            if all_urls {
                // TODO use url = "2.2.2";
                entries.append(
                    &mut URL::find(None, None, None)
                        .await
                        .into_par_iter()
                        .map(|t| t.url)
                        .collect(),
                )
            };

            let mut key_vals = HashMap::new();
            // key_vals.insert("$domain".to_string(), domains);

            for entry in entries {
                key_vals.insert("$$", vec![entry.clone()]);
                // Run commands and run closure for each extracted struct
                tools::run_script(&key_vals, &script_name)
                    .extract_for_each(|t: Host| async {
                        t.update().await;
                    })
                    .await
                    .extract_for_each(|mut t: Sub| async {
                        t.scope = entry.clone();
                        t.update().await;
                    })
                    .await
                    .extract_for_each(|mut t: URL| async {
                        t.sub = entry.clone();
                        t.update().await;
                    })
                    .await;
            }
        }
    }
}
