use super::{Host, Model, URL};
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
    #[derive(Clone,Debug, Serialize, Deserialize, StructOpt,PartialEq, Eq)]
    pub enum SubType {
        IP,
        Domain,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, StructOpt, PartialEq, Eq)]
pub struct Sub {
    #[structopt(short, long)]
    pub asset: String, // id

    #[structopt(short, long)]
    pub scope: String,

    #[structopt(short, long,possible_values = &SubType::variants(),case_insensitive = true)]
    pub ty: Option<SubType>,

    #[structopt(long)]
    pub host: Option<String>,

    #[structopt(short, long)]
    pub urls: Vec<String>,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}
#[async_trait]
impl Model for Sub {
    fn ident() -> String {
        String::from("Sub")
    }

    fn new(id: String, parent: String) -> Self {
        Self {
            asset: id,
            scope: parent,
            host: None,
            ty: None,
            urls: vec![],
            update: Some(DateTime::now()),
        }
    }

    fn id_query(&self) -> Document {
        doc! {"asset":self.asset.clone(),"scope":self.scope.clone()}
    }

    async fn merge(mut self, mut doc: Self) -> Self {
        for s in &self.urls {
            mongo::insert::<URL>(s.clone(), self.asset.clone()).await;
        }
        self.urls.append(&mut doc.urls);
        self.urls.par_sort();
        self.urls.dedup();

        if let Some(host) = self.host.clone() {
            mongo::insert::<Host>(host, self.asset.clone()).await;
        }

        Self {
            asset: self.asset,
            scope: self.scope,
            host: self.host.or(doc.host),
            ty: self.ty.or(doc.ty),
            urls: self.urls,
            update: Some(DateTime::now()),
        }
    }

    fn regex() -> Regex {
        static PAT: &str =
            r"(?:^|//|\s)((?:[0-9\-a-z]+\.)+[0-9a-z][0-9\-a-z]*[0-9a-z])(?:$|[\D\W])";
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(PAT)
                .multi_line(true)
                .build()
                .unwrap();
        }
        RE.clone()
    }

    fn wordlister(&self) -> Vec<String> {
        self.asset.split('.').map(|w| w.to_string()).collect()
    }
}

impl<'t> From<regex::Captures<'t>> for Sub {
    fn from(cap: regex::Captures<'t>) -> Self {
        Sub {
            asset: cap
                .get(1)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            scope: "".to_string(),
            host: cap.get(2).map_or(None, |m| Some(m.as_str().to_string())),
            ty: Some(SubType::Domain),
            urls: vec![],
            update: None,
        }
    }
}
