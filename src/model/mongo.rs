use mongodb::{
    bson::{doc, DateTime, Document},
    options::ClientOptions,
    Client, Database,
};

// This trait is required to use `try_next()` on the cursor
use clap::arg_enum;
use futures::stream::TryStreamExt;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::{
    model::Alert,
    tools::{self},
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
    pub scopes: Vec<String>, //ListField(ReferenceField('Scope'))

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

    // #[structopt(skip)]
    // updated_at: Option<DateTime>,
    #[structopt(short, long)]
    pub program: String, // Program

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
    pub sub: String, //ReferenceField('Sub', reverse_delete_rule:CASCADE)

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
    input_files: Vec<String>, //ListField(String,Field(max_length:512))

    #[structopt(short, long)]
    output_files: Vec<String>, //ListField(String,Field(max_length:512))

    #[structopt(short, long)]
    program: Option<String>, //ReferenceField('Program')

    #[structopt(short, long)]
    scope: String, //ReferenceField('Scope')

    #[structopt(long)]
    host: String, //ReferenceField('Host')

    #[structopt(short, long)]
    url: String, //ReferenceField('URL')

    #[structopt(long)]
    tech: Vec<String>, //ListField(String,Field(max_length:512))

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
        filter: Option<String>,
        limit: Option<String>,
        sort: Option<String>,
    },
    Script {
        #[structopt(short, long)]
        script: String,
        #[structopt(short, long)]
        domains: Vec<String>,
        #[structopt(short, long)]
        all_scopes: bool,
    },
}

pub async fn get_db() -> Database {
    // Get db_url
    let url = super::get_db_url().await;

    // Parse a connection String, into an options }
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
    // Get a handle to a database.
    // let db = get_db().await;

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
            limit,
            sort,
        } => match ty.as_str() {
            "program" => {
                println!("{:#?}", Program::find(filter, limit, sort).await);
            }
            "scope" => {
                println!("{:#?}", Scope::find(filter, limit, sort).await);
            }
            "sub" => {
                println!("{:#?}", Sub::find(filter, limit, sort).await);
            }
            "host" => {
                println!("{:#?}", Host::find(filter, limit, sort).await);
            }
            "url" => {
                println!("{:#?}", URL::find(filter, limit, sort).await);
            }
            "service" => {
                println!("{:#?}", Service::find(filter, limit, sort).await);
            }
            "tech" => {
                println!("{:#?}", Tech::find(filter, limit, sort).await);
            }
            "job" => {
                format!("I'm not sure about implementing this for now").warn();
            }
            typ => format!("Strut '{}' not found!", typ).error(),
        },

        Subcommand::Script {
            script,
            domains,
            all_scopes,
        } => {
            let host_pattern = r"((?:[0-9\-a-z]+\.)+[a-z]+)(?:$|[\D\W]+)((?:[0-9]{1,3}\.){3}[0-9]{1,3})?(?:$|[\D\W\s])";

            // key_vals for replace with $keywords in commands
            let mut key_vals = std::collections::HashMap::new();

            if all_scopes {
                key_vals.insert(
                    "$domain".to_string(),
                    Scope::find(None, None, None)
                        .await
                        .into_iter()
                        .map(|s| s.asset)
                        .collect(),
                );
            } else {
                key_vals.insert("$domain".to_string(), domains);
            }

            futures::future::join_all(
                futures::future::join_all(
                    tools::run::<Host>(key_vals, script.as_str(), host_pattern)
                        .into_iter()
                        .map(|h| h.update()),
                )
                .await
                .into_iter()
                .flatten()
                .map(|h| h.sub.push()),
            )
            .await;
        }
    }
}
