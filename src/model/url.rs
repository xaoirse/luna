use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Url {
    #[structopt(long)]
    pub url: String,

    #[structopt(long)]
    pub title: Option<String>,

    #[structopt(long)]
    pub status_code: Option<String>,

    #[structopt(long, short)]
    pub response: Option<String>,

    #[structopt(long)]
    pub techs: Vec<Tech>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Url {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a.url == b.url {
            let new = a.update < b.update;

            merge(&mut a.title, &mut b.title, new);
            merge(&mut a.status_code, &mut b.status_code, new);
            merge(&mut a.response, &mut b.response, new);

            a.update = a.update.max(b.update);
            a.start = a.start.min(b.start);

            a.techs.append(&mut b.techs);
            a.techs.par_sort();
            a.techs.dedup_by(Tech::same_bucket);

            true
        } else {
            false
        }
    }
    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.url.contains_opt(&filter.url)
            && self.title.contains_opt(&filter.title)
            && self.response.contains_opt(&filter.response)
            && self.status_code.contains_opt(&filter.status_code)
            && check_date(&self.update, &filter.updated_at)
            && check_date(&self.start, &filter.started_at)
            && (filter.tech_is_none() || self.techs.par_iter().any(|t| t.matches(filter)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.url.to_string(),
            1 => format!(
                "{}
    title: {}
    status code: {}
    response: length:{}
    techs: {}
    update: {}
    start: {}
    ",
                self.url,
                self.title.as_ref().map_or("", |s| s),
                self.status_code.as_ref().map_or("", |s| s),
                self.response.as_ref().map_or(0, |s| s.len()),
                self.techs.len(),
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
                self.start.map_or("".to_string(), |s| s.to_rfc2822()),
            ),
            2 => format!(
                "{}
    title: {}
    status code: {}
    responce: {}
    techs: [
        {}]
    update: {}
    start: {}
    ",
                self.url,
                self.title.as_ref().map_or("", |s| s),
                self.status_code.as_ref().map_or("", |s| s),
                self.response.as_ref().map_or("", |s| s),
                self.techs
                    .iter()
                    .map(|s| s.stringify(0))
                    .collect::<Vec<String>>()
                    .join("\n        "),
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
                self.start.map_or("".to_string(), |s| s.to_rfc2822()),
            ),
            _ => format!("{:#?}", self),
        }
    }

    pub fn sub_asset(&self) -> String {
        todo!()
    }
}

impl Default for Url {
    fn default() -> Self {
        Self {
            url: String::new(),
            title: None,
            status_code: None,
            response: None,
            techs: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
        }
    }
}

impl std::str::FromStr for Url {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Url {
            url: s.to_string(),
            ..Default::default()
        })
    }
}
