use super::Model;
use super::Sub;
use async_trait::async_trait;
use clap::arg_enum;
use lazy_static::lazy_static;
use mongodb::bson::{doc, DateTime, Document};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::database::mongo;

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
#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq)]
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

    #[structopt(short, long)]
    pub subs: Vec<String>,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}

#[async_trait]
impl Model for Scope {
    fn ident() -> String {
        String::from("Scope")
    }

    fn new(id: String, parent: String) -> Self {
        Self {
            asset: id,
            program: parent,
            eligible_bounty: None,
            severity: None,
            ty: None,
            subs: vec![],
            update: Some(DateTime::now()),
        }
    }

    fn id_query(&self) -> Document {
        doc! {"asset":self.asset.clone(),"program":self.program.clone()}
    }

    async fn merge(mut self, mut doc: Self) -> Self {
        for s in &self.subs {
            mongo::insert::<Sub>(s.clone(), self.asset.clone()).await;
        }
        self.subs.append(&mut doc.subs);
        self.subs.par_sort();
        self.subs.dedup();

        Self {
            asset: self.asset,
            program: self.program,
            eligible_bounty: self.eligible_bounty.or(doc.eligible_bounty),
            severity: self.severity.or(doc.severity),
            ty: self.ty.or(doc.ty),
            subs: self.subs,
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
        vec![self.asset.clone()]
    }
}

impl<'t> From<regex::Captures<'t>> for Scope {
    fn from(_cap: regex::Captures<'t>) -> Self {
        todo!()
    }
}
