use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

// I was doubt in Program type but this is matter
// that every scopes are in only one program?
// or one scope can be in multi programs?

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt, PartialEq, Eq, PartialOrd, Ord)]
pub struct Program {
    #[structopt(short, long)]
    pub name: String,

    #[structopt(short, long, case_insensitive = true)]
    pub platform: Option<String>,

    #[structopt(long)]
    pub handle: Option<String>,

    #[structopt(short, long)]
    pub typ: Option<String>,

    #[structopt(short, long)]
    pub url: Option<String>,

    #[structopt(short, long)]
    pub icon: Option<String>,

    #[structopt(short, long)]
    pub bounty: Option<String>,

    #[structopt(long, case_insensitive = true)]
    pub state: Option<String>,

    #[structopt(short, long)]
    pub scopes: Vec<Scope>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Program {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if !a.name.is_empty() && a.name.to_lowercase() == b.name.to_lowercase() {
            let new = a.update < b.update;

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
            a.scopes.par_sort();
            a.scopes.dedup_by(Scope::same_bucket);

            true
        } else {
            a.scopes.par_sort();
            a.scopes.dedup_by(Scope::same_bucket);

            false
        }
    }

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

    pub fn set_name(&mut self, luna: &Luna) {
        self.scopes.par_iter_mut().for_each(|s| s.set_name(luna));

        if self.name.is_empty() {
            if let Some(program) = self
                .scopes
                .par_iter()
                .find_map_any(|s| luna.program(&s.asset))
            {
                self.name = program.name.clone();
            }
        }
    }

    /*
        google

        google - google.com

        google - google.com
        icon: url
        platform: hackerone,
        type: Private,
        bounty: 500$,
        state: open,
        scopes: 51,
        started at: Sat 6 19 2019
        updated at: Sat 6 19 2019

        google - google.com
        icon: url
        platform: hackerone,
        type: Private,
        bounty: 500$,
        state: open,
        scopes: [
            a.com
            b.com
            c.com
        ],
        started at: Sat 6 19 2019
        updated at: Sat 6 19 2019

        debug
    */

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
                self.scopes.len(),
                self.update.map_or("".to_string(), |s| s.to_rfc2822()),
                self.start.map_or("".to_string(), |s| s.to_rfc2822()),
            ),
            3 => format!(
                "{}  {}
    platform: {}
    type: {}
    handle: {}
    bounty: {}
    icon: {}
    state: {}
    scopes: [
        {}]
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
