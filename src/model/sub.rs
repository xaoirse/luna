use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use super::*;
use crate::model::url::Url;

#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
pub struct Sub {
    #[structopt(long)]
    pub asset: String,

    #[structopt(long = "type", case_insensitive = true)]
    pub typ: Option<String>,

    #[structopt(long)]
    pub hosts: Vec<Host>,

    #[structopt(long)]
    pub urls: Vec<Url>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,

    #[structopt(skip)]
    pub dedup: bool,
}

impl Dedup for Sub {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        let new = a.update < b.update;

        if a.asset.is_empty() {
            a.asset = std::mem::take(&mut b.asset);
        }

        merge(&mut a.typ, &mut b.typ, new);

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.hosts.append(&mut b.hosts);
        a.urls.append(&mut b.urls);
        a.dedup = false;
    }
    fn dedup(&mut self, term: Arc<AtomicBool>) {
        if self.dedup {
            return;
        }
        self.dedup = dedup(&mut self.hosts, term.clone()) && dedup(&mut self.urls, term)
    }
}

impl Sub {
    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        self.asset.contains_opt(&filter.sub)
            && self.typ.contains_opt(&filter.sub_type)
            && (!date
                || (check_date(&self.update, &filter.updated_at)
                    && check_date(&self.start, &filter.started_at)))
            && (filter.host_is_none() || self.hosts.par_iter().any(|h| h.matches(filter, false)))
            && (filter.url_is_none() || self.urls.par_iter().any(|u| u.matches(filter, false)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.asset.to_string(),
            1 => format!(
                "{}
    type: {}
    hosts: {}
    urls: {}
    update: {}
    start: {}
    ",
                self.asset,
                self.typ.as_ref().map_or("", |s| s),
                self.hosts.len(),
                self.urls.len(),
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            2 => format!(
                "{}
    type: {}
    hosts: [{}{}
    urls: [{}{}
    update: {}
    start: {}
    ",
                self.asset,
                self.typ.as_ref().map_or("", |s| s),
                self.hosts
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.hosts.is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
                self.urls
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(1)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.urls.is_empty() { "]" } else { "\n    ]" },
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
}

impl Default for Sub {
    fn default() -> Self {
        Self {
            asset: String::new(),
            typ: None,
            hosts: vec![],
            urls: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
            dedup: false,
        }
    }
}
impl std::str::FromStr for Sub {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Sub {
            asset: s.to_string(),
            ..Default::default()
        })
    }
}

impl PartialEq for Sub {
    fn eq(&self, other: &Self) -> bool {
        if self.asset.is_empty() || other.asset.is_empty() {
            self.urls
                .par_iter()
                .any(|s| other.urls.par_iter().any(|ss| s == ss))
                || self
                    .hosts
                    .par_iter()
                    .any(|h| other.hosts.par_iter().any(|hh| h == hh))
        } else {
            self.asset == other.asset
        }
    }
}

impl Eq for Sub {}
