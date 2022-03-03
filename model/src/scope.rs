use super::*;
use ::url as urlib;
use chrono::{DateTime, Utc};
use cidr::IpCidr;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Serialize, Deserialize, Parser, Clone)]
pub struct Scope {
    #[clap(long)]
    pub asset: ScopeType,

    #[clap(long)]
    pub bounty: Option<String>,

    #[clap(long, ignore_case = true)]
    pub severity: Option<String>,

    #[clap(long)]
    pub subs: Vec<Sub>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub start: Option<DateTime<Utc>>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ScopeType {
    Domain(urlib::Host),
    #[serde(with = "serde_cidr")]
    Cidr(IpCidr),
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
        } else if let Ok(cidr) = s.parse::<cidr::IpCidr>() {
            Ok(Self::Cidr(cidr))
        } else if let Ok(d) = urlib::Host::parse(s) {
            Ok(Self::Domain(d))
        } else {
            Err("Invalid scope".to_string())
        }
    }
}

impl EqExt for ScopeType {
    fn contains_opt(&self, regex: &Option<Regex>) -> bool {
        match (self, regex) {
            (Self::Domain(d), Some(re)) => re.captures(&d.to_string()).is_some(),
            (Self::Cidr(c), Some(re)) => re.captures(&c.to_string()).is_some(),
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
    }

    fn is_empty(&self) -> bool {
        self.asset == ScopeType::Empty && self.subs.is_empty()
    }
}

impl Scope {
    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        self.asset.contains_opt(&filter.scope)
            && self.bounty.contains_opt(&filter.scope_bounty)
            && self.severity.contains_opt(&filter.scope_severity)
            && (!date
                || (check_date(&self.update, &filter.updated_at)
                    && check_date(&self.start, &filter.started_at)))
            && (filter.sub_is_none() || self.subs.par_iter().any(|s| s.matches(filter, false)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.asset.to_string(),
            1 => format!(
                "{}
    bounty: {}
    severity: {}
    subs: {}
    urls: {}
    hosts: {}
    update: {}
    start: {}
    ",
                self.asset,
                self.bounty.as_ref().map_or("", |s| s),
                self.severity.as_ref().map_or("", |s| s),
                self.subs.iter().filter(|p| !p.asset.is_empty()).count(),
                self.subs.iter().flat_map(|s| &s.urls).count(),
                self.subs.iter().flat_map(|s| &s.hosts).count(),
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
    urls: {}
    hosts: {}
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
                self.subs.iter().flat_map(|s| &s.urls).count(),
                self.subs.iter().flat_map(|s| &s.hosts).count(),
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
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.asset.cmp(&other.asset)
    }
}

impl PartialOrd for Scope {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
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

mod test {

    #[test]
    fn test_eq() {
        use super::*;

        let a = Scope {
            subs: vec![Sub {
                urls: vec![Url::from_str("http://a.a").unwrap()],
                ..Default::default()
            }],
            ..Default::default()
        };
        let b = Scope {
            subs: vec![
                Sub {
                    urls: vec![
                        Url::from_str("http://a.a").unwrap(),
                        Url::from_str("http://a.a").unwrap(),
                    ],
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
