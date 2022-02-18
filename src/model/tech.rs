use super::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone)]
pub struct Tech {
    #[structopt(short, long)]
    pub name: String,

    #[structopt(short, long)]
    pub version: Option<String>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}
impl Tech {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a == b {
            let new = a.update < b.update;

            a.update = a.update.max(b.update);
            a.start = a.start.min(b.start);

            merge(&mut a.version, &mut b.version, new);

            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.name.contains_opt(&filter.tech) && self.version.contains_opt(&filter.tech_version)
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.clone(),
            1 => format!("{} {}", self.name, self.version.as_ref().map_or("", |s| s)),
            2 => format!(
                "{} {}
    Update: {}
    Start: {}
    ",
                self.name,
                self.version.as_ref().map_or("", |s| s),
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

impl Default for Tech {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: None,
            update: Some(Utc::now()),
            start: Some(Utc::now()),
        }
    }
}

impl std::str::FromStr for Tech {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tech {
            name: s.to_string(),
            ..Default::default()
        })
    }
}

impl Ord for Tech {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl PartialOrd for Tech {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Tech {
    fn eq(&self, other: &Self) -> bool {
        self.name.to_lowercase() == other.name.to_lowercase()
    }
}

impl Eq for Tech {}
