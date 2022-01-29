use super::*;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Host {
    #[structopt(short, long)]
    pub ip: String,

    #[structopt(long)]
    pub services: Vec<Service>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
}

impl Host {
    fn new() -> Self {
        Self {
            ip: "".to_string(),
            services: vec![],
            update: None,
        }
    }

    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a.ip == b.ip {
            a.update = a.update.max(b.update);

            a.services.append(&mut b.services);
            a.services.dedup_by(Service::same_bucket);
            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &Filter) -> bool {
        filter
            .ip
            .as_ref()
            .map_or(true, |pat| self.ip.to_lowercase().contains(pat))
            && (filter.port.is_none() && filter.service_name.is_none()
                || self.services.iter().any(|s| s.matches(filter)))
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
}

impl std::str::FromStr for Host {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut host = Self::new();
        host.ip = s.to_string();
        host.update = Some(Utc::now());
        Ok(host)
    }
}

impl<'t> From<regex::Captures<'t>> for Host {
    fn from(cap: regex::Captures<'t>) -> Self {
        Host {
            ip: cap
                .get(2)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            services: vec![],
            update: Some(Utc::now()),
        }
    }
}
