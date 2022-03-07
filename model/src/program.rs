use super::*;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Parser)]
pub struct Program {
    #[clap(long)]
    pub name: String,

    #[clap(long, ignore_case = true)]
    pub platform: Option<String>,

    #[clap(long)]
    pub handle: Option<String>,

    #[clap(long = "type", ignore_case = true)]
    pub typ: Option<String>,

    #[clap(long)]
    pub url: Option<String>,

    #[clap(long)]
    pub icon: Option<String>,

    #[clap(long)]
    pub bounty: Option<String>,

    #[clap(long, ignore_case = true)]
    pub state: Option<String>,

    #[clap(long)]
    pub scopes: Vec<Scope>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub start: Option<DateTime<Utc>>,
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

        let mut i = b.scopes.len();
        while i > 0 {
            i -= 1;

            let b = b.scopes.swap_remove(i);
            if let Some(a) = a.scopes.iter_mut().find(|a| &&b == a) {
                Scope::same(b, a);
            } else {
                a.scopes.push(b);
            }
        }
    }

    fn dedup(&mut self, term: Arc<AtomicBool>) {
        dedup(&mut self.scopes, term);
    }

    fn is_empty(&self) -> bool {
        self.name.is_empty() && self.scopes.is_empty()
    }
    fn no_name(&self) -> bool {
        self.name.is_empty()
    }
}

impl Program {
    pub fn same(mut b: Self, a: &mut Self) {
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

        for b in b.scopes {
            if let Some(a) = a.scopes.iter_mut().find(|a| &&b == a) {
                Scope::same(b, a);
            } else {
                a.scopes.push(b);
            }
        }
    }

    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        self.name.contains_opt(&filter.program)
            && self.platform.contains_opt(&filter.program_platform)
            && self.handle.contains_opt(&filter.program_handle)
            && self.typ.contains_opt(&filter.program_type)
            && self.url.contains_opt(&filter.program_url)
            && self.icon.contains_opt(&filter.program_icon)
            && self.bounty.contains_opt(&filter.program_bounty)
            && self.state.contains_opt(&filter.program_state)
            && (!date
                || (check_date(&self.update, &filter.updated_at)
                    && check_date(&self.start, &filter.started_at)))
            && (filter.scope_is_none() || self.scopes.par_iter().any(|s| s.matches(filter, false)))
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

impl Ord for Program {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Program {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        if self.name.is_empty() || other.name.is_empty() {
            self.scopes
                .iter()
                .any(|s| other.scopes.iter().any(|ss| s == ss))
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
