use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone)]
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
            a.techs.par_sort();
            a.techs.dedup_by(Tech::same_bucket);

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
                "{} [{}]",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
            ),
            2 => format!(
                "{} [{}] [{}]",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
                self.title.as_ref().map_or("", |s| s),
            ),
            3 => format!(
                "{} [{}]
    title: {}
    response: length:{}
    techs: {}
    update: {}
    start: {}
    ",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
                self.title.as_ref().map_or("", |s| s),
                self.response
                    .as_ref()
                    .map_or("N".to_string(), |s| s.len().to_string()),
                self.techs.len(),
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            4 => format!(
                "{} [{}]
    title: {}
    responce: {}
    techs: [{}{}
    update: {}
    start: {}
    ",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
                self.title.as_ref().map_or("", |s| s),
                self.response.as_ref().map_or("", |s| s),
                self.techs
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.techs.is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
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

impl Ord for Url {
    fn cmp(&self, other: &Self) -> Ordering {
        self.url.cmp(&other.url)
    }
}

impl PartialOrd for Url {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Url {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Eq for Url {}
