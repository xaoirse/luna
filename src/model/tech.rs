use super::Model;
use async_trait::async_trait;
use lazy_static::lazy_static;
use mongodb::bson::{doc, Document};
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq)]
pub struct Tech {
    #[structopt(short, long)]
    pub name: String,

    #[structopt(short, long)]
    pub version: String,
}

#[async_trait]
impl Model for Tech {
    fn ident() -> String {
        String::from("Tech")
    }

    fn new(id: String, parent: String) -> Self {
        Self {
            name: id,
            version: parent,
        }
    }

    fn id_query(&self) -> Document {
        doc! {"name":self.name.clone(),"version":self.version.clone()}
    }

    // TODO implement update query for changing version
    async fn merge(mut self, _doc: Self) -> Self {
        Self {
            name: self.name,
            version: self.version,
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

impl<'t> From<regex::Captures<'t>> for Tech {
    fn from(_cap: regex::Captures<'t>) -> Self {
        todo!()
    }
}
