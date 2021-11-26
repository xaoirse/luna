use super::{Model, Service};
use async_trait::async_trait;
use lazy_static::lazy_static;
use mongodb::bson::{doc, DateTime, Document};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::database::mongo;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Host {
    #[structopt(short, long)]
    pub ip: String,

    #[structopt(long)]
    pub services: Vec<String>,

    #[structopt(short, long)]
    pub sub: String,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}

#[async_trait]
impl Model for Host {
    fn ident() -> String {
        String::from("Host")
    }

    fn new(id: String, parent: String) -> Self {
        Self {
            ip: id,
            sub: parent,
            services: vec![],
            update: Some(DateTime::now()),
        }
    }

    fn id_query(&self) -> Document {
        doc! {"ip":self.ip.clone()}
    }

    async fn merge(mut self, mut doc: Self) -> Self {
        for s in &self.services {
            mongo::insert::<Service>(s.clone(), "".to_string()).await;
        }
        self.services.append(&mut doc.services);
        self.services.par_sort();
        self.services.dedup();

        Self {
            ip: self.ip,
            sub: self.sub,
            services: self.services,
            update: Some(DateTime::now()),
        }
    }

    fn regex() -> Regex {
        static PAT: &str = r"(?:^|//|\s|\b)((?:[0-9\-a-z]+\.)+[0-9a-z][0-9\-a-z]*[0-9a-z])[\D\W]*((?:[0-9]{1,3}\.){3}[0-9]{1,3})(?:$|[\D\W\s])";
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(PAT)
                .multi_line(true)
                .build()
                .unwrap();
        }
        RE.clone()
    }

    fn wordlister(&self) -> Vec<String> {
        vec![self.ip.clone()]
    }
}

impl<'t> From<regex::Captures<'t>> for Host {
    fn from(cap: regex::Captures<'t>) -> Self {
        Host {
            sub: cap
                .get(1)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            ip: cap
                .get(2)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            services: vec![],
            update: None,
        }
    }
}
