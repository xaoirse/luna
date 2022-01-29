use super::*;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Default, Clone, Debug, Serialize, Deserialize, StructOpt, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Sub {
    #[structopt(short, long)]
    pub asset: String,

    #[structopt(short, long,possible_values = &["IP","Domain"] ,case_insensitive= true)]
    pub typ: Option<String>,

    #[structopt(short = "i", long)]
    pub hosts: Vec<Host>,

    #[structopt(short, long)]
    pub urls: Vec<Url>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
}

impl Sub {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if !a.asset.is_empty() && a.asset == b.asset {
            let new = a.update < b.update;

            merge(&mut a.typ, &mut b.typ, new);

            a.update = a.update.max(b.update);

            a.hosts.append(&mut b.hosts);
            a.hosts.sort();
            a.hosts.dedup_by(Host::same_bucket);

            a.urls.append(&mut b.urls);
            a.urls.sort();
            a.urls.dedup_by(Url::same_bucket);
            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &Filter) -> bool {
        filter
            .sub
            .as_ref()
            .map_or(true, |pat| self.asset.to_lowercase().contains(pat))
            && (filter.ip.is_none() && filter.port.is_none() && filter.service_name.is_none()
                || self.hosts.iter().any(|h| h.matches(filter)))
            && (filter.url.is_none()
                && filter.title.is_none()
                && filter.status_code.is_none()
                && filter.content_type.is_none()
                && filter.content_length.is_none()
                && filter.tech.is_none()
                && filter.tech_version.is_none()
                || self.urls.iter().any(|u| u.matches(filter)))
    }

    pub fn set_name(&mut self, luna: &Luna) {
        if let Some(url) = self.urls.first() {
            self.asset = url.sub_asset();
        } else {
            for i in 0..self.hosts.len() {
                if let Some(sub) = luna.sub(&self.hosts[i].ip) {
                    self.asset = sub.asset.clone();
                    break;
                }
            }
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
}

impl std::str::FromStr for Sub {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sub = Self::new();
        sub.asset = s.to_string();
        sub.update = Some(Utc::now());
        Ok(sub)
    }
}

// impl<'t> From<regex::Captures<'t>> for Sub {
//     fn from(cap: regex::Captures<'t>) -> Self {
//         Sub {
//             asset: cap
//                 .get(1)
//                 .map_or("".to_string(), |m| m.as_str().to_string()),

//             host: vec![cap.get(2).map_or(None, |m| Some(m.as_str().to_string()))],
//             typ: Some("domain".to_string()),
//             urls: vec![],
//             update: None,
//         }
//     }
// }
