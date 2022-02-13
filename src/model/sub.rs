use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use structopt::StructOpt;

#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
pub struct Sub {
    #[structopt(short, long)]
    pub asset: String,

    #[structopt(short, long, case_insensitive = true)]
    pub typ: Option<String>,

    #[structopt(short = "i", long)]
    pub hosts: Vec<Host>,

    #[structopt(short, long)]
    pub urls: Vec<Url>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Sub {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if !a.asset.is_empty() && a.asset == b.asset {
            let new = a.update < b.update;

            merge(&mut a.typ, &mut b.typ, new);

            a.update = a.update.max(b.update);
            a.start = a.start.min(b.start);

            a.hosts.append(&mut b.hosts);
            a.hosts.par_sort();
            a.hosts.dedup_by(Host::same_bucket);

            a.urls.append(&mut b.urls);
            a.urls.par_sort();
            a.urls.dedup_by(Url::same_bucket);

            true
        } else {
            a.hosts.par_sort();
            a.hosts.dedup_by(Host::same_bucket);

            a.urls.par_sort();
            a.urls.dedup_by(Url::same_bucket);

            false
        }
    }

    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.asset.contains_opt(&filter.sub)
            && self.typ.contains_opt(&filter.sub_type)
            && check_date(&self.update, &filter.updated_at)
            && check_date(&self.start, &filter.started_at)
            && (filter.host_is_none() || self.hosts.par_iter().any(|h| h.matches(filter)))
            && (filter.url_is_none() || self.urls.par_iter().any(|u| u.matches(filter)))
    }

    pub fn set_name(&mut self, luna: &Luna) {
        if let Some(url) = self.urls.first() {
            self.asset = url.sub_asset();
        } else if let Some(sub) = self.hosts.par_iter_mut().find_map_any(|h| luna.sub(&h.ip)) {
            self.asset = sub.asset.clone();
        }
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
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
                self.start.map_or("".to_string(), |s| s.to_rfc2822()),
            ),
            2 => format!(
                "{}
    type: {}
    hosts: [{}]
    urls: [{}]
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
                self.urls
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
                self.start.map_or("".to_string(), |s| s.to_rfc2822()),
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

impl Ord for Sub {
    fn cmp(&self, other: &Self) -> Ordering {
        self.asset.cmp(&other.asset)
    }
}

impl PartialOrd for Sub {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Sub {
    fn eq(&self, other: &Self) -> bool {
        self.asset == other.asset
    }
}

impl Eq for Sub {}
