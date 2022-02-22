use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
pub struct Program {
    #[structopt(long)]
    pub name: String,

    #[structopt(long, case_insensitive = true)]
    pub platform: Option<String>,

    #[structopt(long)]
    pub handle: Option<String>,

    #[structopt(long = "type", case_insensitive = true)]
    pub typ: Option<String>,

    #[structopt(long)]
    pub url: Option<String>,

    #[structopt(long)]
    pub icon: Option<String>,

    #[structopt(long)]
    pub bounty: Option<String>,

    #[structopt(long, case_insensitive = true)]
    pub state: Option<String>,

    #[structopt(long)]
    pub scopes: Vec<Scope>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,

    #[structopt(skip)]
    pub dedup: bool,
}

impl Dedup for Program {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        let new = a.update < b.update;

        if a.name.is_empty() {
            a.name = std::mem::take(&mut b.name);
        }

        merge(&mut a.platform, &mut b.platform, new);
        merge(&mut a.handle, &mut b.handle, new);
        merge(&mut a.typ, &mut b.typ, new);
        merge(&mut a.url, &mut b.url, new);
        merge(&mut a.icon, &mut b.icon, new);
        merge(&mut a.bounty, &mut b.bounty, new);
        merge(&mut a.state, &mut b.state, new);

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.scopes.append(&mut b.scopes);
        a.dedup = false;
    }
    fn dedup(&mut self, term: Arc<AtomicBool>) {
        if self.dedup {
            return;
        }
        dedup(&mut self.scopes, term);
        self.dedup = true
    }
}

impl Program {
    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.name.contains_opt(&filter.program)
            && self.platform.contains_opt(&filter.program_platform)
            && self.handle.contains_opt(&filter.program_handle)
            && self.typ.contains_opt(&filter.program_type)
            && self.url.contains_opt(&filter.program_url)
            && self.icon.contains_opt(&filter.program_icon)
            && self.bounty.contains_opt(&filter.program_bounty)
            && self.state.contains_opt(&filter.program_state)
            && check_date(&self.update, &filter.updated_at)
            && check_date(&self.start, &filter.started_at)
            && (filter.scope_is_none() || self.scopes.par_iter().any(|s| s.matches(filter)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!("{}  {} ", self.name, self.url.as_ref().map_or("", |s| s)),
            2 => format!(
                "{}  {}
    platform: {}
    type: {}
    handle: {}
    bounty: {}
    icon: {}
    state: {}
    scopes: {}
    subs: {}
    urls: {}
    hosts: {}
    update: {}
    start: {}
    ",
                self.name,
                self.url.as_ref().map_or("", |s| s),
                self.platform.as_ref().map_or("", |s| s),
                self.typ.as_ref().map_or("", |s| s),
                self.handle.as_ref().map_or("", |s| s),
                self.bounty.as_ref().map_or("", |s| s),
                self.icon.as_ref().map_or("", |s| s),
                self.state.as_ref().map_or("", |s| s),
                self.scopes
                    .iter()
                    .filter(|p| p.asset != ScopeType::Empty)
                    .count(),
                self.scopes.iter().flat_map(|s| &s.subs).count(),
                self.scopes
                    .iter()
                    .flat_map(|s| &s.subs)
                    .flat_map(|s| &s.urls)
                    .count(),
                self.scopes
                    .iter()
                    .flat_map(|s| &s.subs)
                    .flat_map(|s| &s.hosts)
                    .count(),
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            3 => format!(
                "{}  {}
    platform: {}
    type: {}
    handle: {}
    bounty: {}
    icon: {}
    state: {}
    scopes: [{}{}
    subs: {}
    urls: {}
    hosts: {}
    update: {}
    start: {}
    ",
                self.name,
                self.url.as_ref().map_or("", |s| s),
                self.platform.as_ref().map_or("", |s| s),
                self.typ.as_ref().map_or("", |s| s),
                self.handle.as_ref().map_or("", |s| s),
                self.bounty.as_ref().map_or("", |s| s),
                self.icon.as_ref().map_or("", |s| s),
                self.state.as_ref().map_or("", |s| s),
                self.scopes
                    .iter()
                    .filter(|p| p.asset != ScopeType::Empty)
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self
                    .scopes
                    .iter()
                    .filter(|p| p.asset != ScopeType::Empty)
                    .count()
                    == 0
                {
                    "]"
                } else {
                    "\n    ]"
                },
                self.scopes.iter().flat_map(|s| &s.subs).count(),
                self.scopes
                    .iter()
                    .flat_map(|s| &s.subs)
                    .flat_map(|s| &s.urls)
                    .count(),
                self.scopes
                    .iter()
                    .flat_map(|s| &s.subs)
                    .flat_map(|s| &s.hosts)
                    .count(),
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

impl Default for Program {
    fn default() -> Self {
        Self {
            name: String::new(),
            url: None,
            platform: None,
            typ: None,
            state: None,
            icon: None,
            bounty: None,
            handle: None,
            scopes: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
            dedup: false,
        }
    }
}

impl std::str::FromStr for Program {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Program {
            name: s.to_string(),
            ..Default::default()
        })
    }
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        if self.name.is_empty() || other.name.is_empty() {
            self.scopes
                .par_iter()
                .any(|s| other.scopes.par_iter().any(|ss| s == ss))
        } else {
            self.name.to_lowercase() == other.name.to_lowercase()
        }
    }
}

impl Eq for Program {}

#[allow(unused_imports)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_eq() {
        let a = Program {
            scopes: vec![Scope {
                subs: vec![Sub::from_str("1").unwrap(), Sub::from_str("2").unwrap()],
                ..Default::default()
            }],
            ..Default::default()
        };
        let b = Program {
            scopes: vec![Scope {
                subs: vec![Sub::from_str("3").unwrap(), Sub::from_str("1").unwrap()],
                ..Default::default()
            }],
            ..Default::default()
        };

        assert_eq!(a, b)
    }
}
