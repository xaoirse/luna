use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone)]
pub struct Scope {
    #[structopt(long)]
    pub asset: ScopeType,

    #[structopt(long)]
    pub bounty: Option<String>,

    #[structopt(long, case_insensitive = true)]
    pub severity: Option<String>,

    #[structopt(long)]
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

impl Default for ScopeType {
    fn default() -> Self {
        Self::Empty
    }
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
        } else if s.parse::<cidr::IpCidr>().is_ok() {
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

impl Dedup for Scope {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        let new = a.update < b.update;

        if a.asset == ScopeType::Empty {
            a.asset = std::mem::take(&mut b.asset);
        }

        merge(&mut a.bounty, &mut b.bounty, new);
        merge(&mut a.severity, &mut b.severity, new);

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.subs.append(&mut b.subs);

        a.dedup();
    }
    fn dedup(&mut self) {
        dedup(&mut self.subs);
    }
}

impl Scope {
    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.asset.contains_opt(&filter.scope)
            && self.bounty.contains_opt(&filter.scope_bounty)
            && self.severity.contains_opt(&filter.scope_severity)
            && check_date(&self.update, &filter.updated_at)
            && check_date(&self.start, &filter.started_at)
            && (filter.sub_is_none() || self.subs.par_iter().any(|s| s.matches(filter)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.asset.to_string(),
            1 => format!(
                "{}
    bounty: {}
    severity: {}
    subs: {}
    update: {}
    start: {}
    ",
                self.asset,
                self.bounty.as_ref().map_or("", |s| s),
                self.severity.as_ref().map_or("", |s| s),
                self.subs.iter().filter(|p| !p.asset.is_empty()).count(),
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            2 => format!(
                "{},
    bounty: {},
    severity: {}
    subs: [{}{}
    update: {}
    start: {}
    ",
                self.asset,
                self.bounty.as_ref().map_or("", |s| s),
                self.severity.as_ref().map_or("", |s| s),
                self.subs
                    .iter()
                    .filter(|p| !p.asset.is_empty())
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.subs.iter().filter(|p| !p.asset.is_empty()).count() == 0 {
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

impl Default for Scope {
    fn default() -> Self {
        Self {
            asset: ScopeType::Empty,
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

impl Ord for Scope {
    fn cmp(&self, other: &Self) -> Ordering {
        self.asset.cmp(&other.asset)
    }
}

impl PartialOrd for Scope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        if self.asset == ScopeType::Empty || other.asset == ScopeType::Empty {
            self.subs
                .par_iter()
                .any(|s| other.subs.par_iter().any(|ss| s == ss))
        } else {
            self.asset == other.asset
        }
    }
}

impl Eq for Scope {}

#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::model::url::Url;
    #[test]
    fn test_eq() {
        let a = Scope {
            subs: vec![Sub {
                urls: vec![Url::from_str("a").unwrap()],
                ..Default::default()
            }],
            ..Default::default()
        };
        let b = Scope {
            subs: vec![
                Sub {
                    urls: vec![Url::from_str("a").unwrap(), Url::from_str("b").unwrap()],
                    ..Default::default()
                },
                Sub {
                    asset: "b".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(a, b)
    }
}
