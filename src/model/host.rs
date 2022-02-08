use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Debug, Default, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
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

    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.ip.contains_opt(&filter.ip)
            && check_date(&self.update, &filter.days_before)
            && (filter.service_is_none() || self.services.par_iter().any(|s| s.matches(filter)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0..=1 => self.ip.to_string(),
            2 => format!(
                "{},
    services: {},
    update: {}
    ",
                self.ip,
                self.services.len(),
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
            ),
            3 => format!(
                "{},
    services: [
        {}],
    update: {}
    ",
                self.ip,
                self.services
                    .iter()
                    .map(|s| s.stringify(1))
                    .collect::<Vec<String>>()
                    .join(",\n        "),
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
            ),
            _ => format!("{:#?}", self),
        }
    }
}

impl std::str::FromStr for Host {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Host {
            ip: s.to_string(),
            update: Some(Utc::now()),
            ..Default::default()
        })
    }
}
