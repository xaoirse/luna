use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

// I was doubt in Program type but this is matter
// that every scopes are in only one program?
// or one scope can be in multi programs?

#[derive(
    Debug, Clone, Serialize, Deserialize, StructOpt, PartialEq, Eq, PartialOrd, Ord, Default,
)]
pub struct Program {
    #[structopt(short, long)]
    pub name: String,

    #[structopt(short, long, case_insensitive = true)]
    pub platform: Option<String>,

    #[structopt(long)]
    pub handle: Option<String>,

    #[structopt(short, long,possible_values =&["public","private"])]
    pub typ: Option<String>,

    #[structopt(short, long)]
    pub url: Option<String>,

    #[structopt(short, long)]
    pub icon: Option<String>,

    #[structopt(short, long)]
    pub bounty: Option<String>,

    #[structopt(long,possible_values=&["open","closed"],case_insensitive = true)]
    pub state: Option<String>,

    #[structopt(short, long)]
    pub scopes: Vec<Scope>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub started_at: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
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
            a.started_at = a.started_at.min(b.started_at);

            a.scopes.append(&mut b.scopes);
            a.scopes.par_sort();
            a.scopes.dedup_by(Scope::same_bucket);

            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &Filter) -> bool {
        filter
            .program
            .as_ref()
            .map_or(true, |pat| self.name.to_lowercase().contains(pat))
            && has(&self.platform, &filter.program_platform)
            && has(&self.typ, &filter.program_type)
            && (filter.program_bounty.is_none() || filter.program_bounty == self.bounty)
            && (filter.program_state.is_none() || filter.program_state == self.state)
            && ((filter.scope.is_none()
                && filter.scope_type.is_none()
                && filter.scope_bounty.is_none()
                && filter.sub.is_none()
                && filter.ip.is_none()
                && filter.port.is_none()
                && filter.service_name.is_none()
                && filter.url.is_none()
                && filter.title.is_none()
                && filter.status_code.is_none()
                && filter.content_type.is_none()
                && filter.content_length.is_none()
                && filter.tech.is_none()
                && filter.tech_version.is_none())
                || self.scopes.par_iter().any(|s| s.matches(filter)))
    }

    pub fn set_name(&mut self, luna: &Luna) {
        self.scopes.par_iter_mut().for_each(|s| s.set_name(luna));

        if self.name.is_empty() {
            if let Some(program) = self
                .scopes
                .par_iter_mut()
                .find_map_any(|s| luna.program(&s.asset))
            {
                self.name = program.name.clone();
            }
        }
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => String::new(),
            _ => "".to_string(),
        }
    }
}

impl std::str::FromStr for Program {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Program {
            name: s.to_string(),
            update: Some(Utc::now()),
            ..Default::default()
        })
    }
}

/*

    google - google.com
    icon: url
    platform: hackerone,
    type: Private,
    bounty: 500$,
    state: open,
    scopes: 51,
    started at: Sat 6 19 2019
    updated at: Sat 6 19 2019

*/
