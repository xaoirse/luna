use super::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone)]
pub struct Tag {
    #[structopt(short, long)]
    pub name: String,

    #[structopt(long)]
    pub severity: Option<String>,

    #[structopt(long)]
    pub values: Vec<String>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}
impl Tag {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a == b {
            let new = a.update < b.update;

            a.update = a.update.max(b.update);
            a.start = a.start.min(b.start);

            merge(&mut a.severity, &mut b.severity, new);

            a.values.append(&mut b.values);
            a.values.par_sort();
            a.values.dedup();

            true
        } else {
            a.values.par_sort();
            a.values.dedup();

            false
        }
    }

    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.name.contains_opt(&filter.tag)
            && self.severity.contains_opt(&filter.tag_severity)
            && self
                .values
                .iter()
                .any(|v| v.contains_opt(&filter.tag_value))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!(
                "{} [{}]",
                self.name,
                self.severity.as_ref().map_or("", |s| s),
            ),
            2 => format!(
                "{} [{}]
    values:[{}{}
    Update: {}
    Start: {}
    ",
                self.name,
                self.severity.as_ref().map_or("", |s| s),
                self.values
                    .iter()
                    .map(|s| format!("\n        {}", s))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.values.is_empty() {
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
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            name: String::new(),
            severity: None,
            values: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
        }
    }
}

impl std::str::FromStr for Tag {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tag {
            name: s.to_string(),
            ..Default::default()
        })
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name.to_lowercase() == other.name.to_lowercase()
    }
}

impl Eq for Tag {}
