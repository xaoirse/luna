use std::{
    fmt::{self, Display},
    str::FromStr,
};

use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scope {
    #[structopt(short, long)]
    pub asset: ScopeType,

    #[structopt(short, long, case_insensitive = true)]
    pub typ: Option<String>,

    #[structopt(short, long)]
    pub bounty: Option<String>,

    #[structopt(long, case_insensitive = true)]
    pub severity: Option<String>,

    #[structopt(short, long)]
    pub subs: Vec<Sub>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ScopeType {
    Domain(String),
    Cidr(String),
    Empty,
}
impl Display for ScopeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(s) => write!(f, "{}", s),
            Self::Cidr(s) => write!(f, "{}", s),
            Self::Empty => write!(f, ""),
        }
    }
}
impl FromStr for ScopeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::Empty)
        } else if s.starts_with('1') {
            Ok(Self::Cidr(s.to_string()))
        } else {
            Ok(Self::Domain(s.to_string()))
        }
    }
}
impl EqExt for ScopeType {
    fn contains_opt(&self, regex: &Option<Regex>) -> bool {
        match (self, regex) {
            (Self::Domain(text), Some(re)) => re.captures(text).is_some(),
            (Self::Cidr(text), Some(re)) => re.captures(text).is_some(),
            (_, None) => true,
            (Self::Empty, _) => false,
        }
    }
}

impl Scope {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a.asset != ScopeType::Empty && a.asset == b.asset {
            let new = a.update < b.update;

            merge(&mut a.typ, &mut b.typ, new);
            merge(&mut a.bounty, &mut b.bounty, new);
            merge(&mut a.severity, &mut b.severity, new);

            a.update = a.update.max(b.update);
            a.start = a.start.min(b.start);

            a.subs.append(&mut b.subs);
            a.subs.par_sort();
            a.subs.dedup_by(Sub::same_bucket);
            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.asset.contains_opt(&filter.scope)
            && self.typ.contains_opt(&filter.scope_type)
            && self.bounty.contains_opt(&filter.scope_bounty)
            && self.severity.contains_opt(&filter.scope_severity)
            && check_date(&self.update, &filter.updated_at)
            && check_date(&self.start, &filter.started_at)
            && (filter.sub_is_none() || self.subs.par_iter().any(|s| s.matches(filter)))
    }

    pub fn set_name(&mut self, luna: &Luna) {
        self.subs
            .par_iter_mut()
            .filter(|s| s.asset.is_empty())
            .for_each(|s| s.set_name(luna));

        if self.asset == ScopeType::Empty {
            if let Some(scope) = self
                .subs
                .par_iter_mut()
                .find_map_any(|s| luna.scope(&s.asset))
            {
                self.asset = scope.asset.clone();
            }
        }
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.asset.to_string(),
            1 => format!(
                "{}
    type: {}
    bounty: {}
    severity: {}
    subs: {}
    update: {}
    start: {}
    ",
                self.asset,
                self.typ.as_ref().map_or("", |s| s),
                self.bounty.as_ref().map_or("", |s| s),
                self.severity.as_ref().map_or("", |s| s),
                self.subs.len(),
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
                self.start.map_or("".to_string(), |s| s.to_rfc2822()),
            ),
            2 => format!(
                "{},
    type: {},
    bounty: {},
    severity: {}
    subs: [
        {}]
    update: {}
    start: {}
    ",
                self.asset,
                self.typ.as_ref().map_or("", |s| s),
                self.bounty.as_ref().map_or("", |s| s),
                self.severity.as_ref().map_or("", |s| s),
                self.subs
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
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            asset: ScopeType::Empty,
            typ: None,
            severity: None,
            bounty: None,
            subs: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
        }
    }
}

impl std::str::FromStr for Scope {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Scope {
            asset: ScopeType::from_str(s).unwrap(),
            ..Default::default()
        })
    }
}
