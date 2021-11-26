use super::{Model, Tech};
use async_trait::async_trait;
use lazy_static::lazy_static;
use mongodb::bson::{doc, DateTime, Document};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::database::mongo;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq)]
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

    #[structopt(short, long)]
    pub techs: Vec<String>,

    #[structopt(skip)]
    pub update: Option<DateTime>,
}

#[async_trait]
impl Model for URL {
    fn ident() -> String {
        String::from("URL")
    }

    fn new(id: String, parent: String) -> Self {
        Self {
            url: id,
            sub: parent,
            title: None,
            status_code: None,
            content_type: None,
            techs: vec![],
            update: Some(DateTime::now()),
        }
    }

    fn id_query(&self) -> Document {
        doc! {"url":self.url.clone(),"sub":self.sub.clone()}
    }

    async fn merge(mut self, mut doc: Self) -> Self {
        for s in &self.techs {
            mongo::insert::<Tech>(s.clone(), self.url.clone()).await;
        }
        self.techs.append(&mut doc.techs);
        self.techs.par_sort();
        self.techs.dedup();

        Self {
            url: self.url,
            sub: self.sub,
            title: self.title.or(doc.title),
            status_code: self.status_code.or(doc.status_code),
            content_type: self.content_type.or(doc.content_type),
            techs: self.techs,
            update: Some(DateTime::now()),
        }
    }

    fn regex() -> Regex {
        // TODO scheme is in match 1
        static PAT: &str = r"(\w+)://[-a-zA-Z0-9:@;?&=/%\+\.\*!'\(\),\$_\{\}\^~\[\]`#|]+";
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(PAT)
                .multi_line(true)
                .build()
                .unwrap();
        }
        RE.clone()
    }

    fn wordlister(&self) -> Vec<String> {
        // TODO from param and args in url
        self.url.split('.').map(|w| w.to_string()).collect()
    }
}

impl<'t> From<regex::Captures<'t>> for URL {
    fn from(cap: regex::Captures<'t>) -> Self {
        URL {
            url: cap
                .get(0)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            sub: "".to_string(),
            title: None,
            status_code: None,
            content_type: None,
            techs: vec![],
            update: None,
        }
    }
}
