use super::Model;
use async_trait::async_trait;
use lazy_static::lazy_static;
use mongodb::bson::{doc, Document};
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq)]
pub struct Service {
    #[structopt(long)]
    pub port: String,

    #[structopt(short, long)]
    pub protocol: Option<String>,

    #[structopt(short, long)]
    pub banner: Option<String>,
}

#[async_trait]
impl Model for Service {
    fn ident() -> String {
        String::from("Service")
    }

    fn new(id: String, _parent: String) -> Self {
        Self {
            port: id,
            protocol: None,
            banner: None,
        }
    }

    fn id_query(&self) -> Document {
        doc! {"port":self.port.clone()}
    }

    async fn merge(mut self, doc: Self) -> Self {
        Self {
            port: self.port,
            protocol: self.protocol.or(doc.protocol),
            banner: self.banner.or(doc.banner),
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
        vec![self.port.clone()]
    }
}

impl<'t> From<regex::Captures<'t>> for Service {
    fn from(_cap: regex::Captures<'t>) -> Self {
        todo!()
    }
}
