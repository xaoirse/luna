use chrono::{DateTime, Utc};
use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, Parser)]
pub struct Sub {
    #[clap(long)]
    pub asset: String,

    #[clap(long = "type", ignore_case = true)]
    pub typ: Option<String>,

    #[clap(long)]
    pub hosts: Vec<Host>,

    #[clap(long)]
    pub urls: Vec<Url>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub start: Option<DateTime<Utc>>,
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

        let mut i = b.hosts.len();
        while i > 0 {
            i -= 1;

            let b = b.hosts.swap_remove(i);
            if let Some(a) = a.hosts.iter_mut().find(|a| &&b == a) {
                Host::same(b, a);
            } else {
                a.hosts.push(b);
            }
        }

        let mut i = b.urls.len();
        while i > 0 {
            i -= 1;

            let b = b.urls.swap_remove(i);
            if let Some(a) = a.urls.iter_mut().find(|a| &&b == a) {
                Url::same(b, a);
            } else {
                a.urls.push(b);
            }
        }
    }

    fn dedup(&mut self, term: Arc<AtomicBool>) {
        dedup(&mut self.hosts, term.clone());
        dedup(&mut self.urls, term);
    }

    fn is_empty(&self) -> bool {
        self.asset.is_empty() && self.urls.is_empty() && self.hosts.is_empty()
    }
    fn no_name(&self) -> bool {
        self.asset.is_empty()
    }
}

impl Sub {
    pub fn same(mut b: Self, a: &mut Self) {
        let new = a.update < b.update;

        if a.asset.is_empty() {
            a.asset = std::mem::take(&mut b.asset);
        }

        merge(&mut a.typ, &mut b.typ, new);

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.hosts.par_sort();
        for b in b.hosts {
            if let Ok(i) = a.hosts.binary_search(&b) {
                Host::same(b, &mut a.hosts[i]);
            } else {
                a.hosts.push(b);
            }
        }

        a.urls.par_sort();
        for b in b.urls {
            if let Ok(i) = a.urls.binary_search(&b) {
                Url::same(b, &mut a.urls[i]);
            } else {
                a.urls.push(b);
            }
        }
    }

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
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.asset.cmp(&other.asset)
    }
}

impl PartialOrd for Sub {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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
