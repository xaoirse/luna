use async_trait::async_trait;
use clap::arg_enum;
use lazy_static::*;
use mongodb::bson::{doc, DateTime, Document};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use super::Scope;
use crate::database::mongo;

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
#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq)]
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

    #[structopt(short, long)]
    pub scopes: Vec<String>,

    #[structopt(skip)]
    // started_at: Option<DateTime>,
    pub update: Option<DateTime>,
}

#[async_trait]
impl super::Model for Program {
    fn ident() -> String {
        String::from("Program")
    }

    fn new(id: String, _parent: String) -> Self {
        Program {
            name: id,
            platform: None,
            handle: None,
            ty: None,
            url: None,
            icon: None,
            bounty: None,
            state: None,
            scopes: vec![],
            update: Some(DateTime::now()),
        }
    }

    fn id_query(&self) -> Document {
        doc! {"name":self.name.clone()}
    }

    async fn merge(mut self, mut doc: Self) -> Self {
        for s in &self.scopes {
            mongo::insert::<Scope>(s.clone(), self.name.clone()).await;
        }
        self.scopes.append(&mut doc.scopes);
        self.scopes.par_sort();
        self.scopes.dedup();

        Program {
            name: self.name,
            platform: self.platform.or(doc.platform),
            handle: self.handle.or(doc.handle),
            ty: self.ty.or(doc.ty),
            url: self.url.or(doc.url),
            icon: self.icon.or(doc.icon),
            bounty: self.bounty.or(doc.bounty),
            state: self.state.or(doc.state),
            scopes: self.scopes,
            update: Some(DateTime::now()),
        }
    }

    fn regex() -> Regex {
        static PAT: &str = r"";
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(PAT)
                .multi_line(true)
                .build()
                .unwrap();
        }
        RE.clone()
    }

    fn wordlister(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
}

impl<'t> From<regex::Captures<'t>> for Program {
    fn from(_cap: regex::Captures<'t>) -> Self {
        todo!()
    }
}
