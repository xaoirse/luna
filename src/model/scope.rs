use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Default, Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Scope {
    #[structopt(short, long)]
    pub asset: String,

    #[structopt(short, long,possible_values = &["SingleDomain","WildcardDomain","Mobile","IOS","Android","PC","Windows","Mac","Linux","SourceCode","CIDR"],case_insensitive = true)]
    pub typ: Option<String>,

    #[structopt(short, long)]
    pub bounty: Option<String>,

    #[structopt(long,possible_values = &["Critical","High","Medium","Low","None"],case_insensitive = true)]
    pub severity: Option<String>,

    #[structopt(short, long)]
    pub subs: Vec<Sub>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
}

impl Scope {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if !a.asset.is_empty() && a.asset == b.asset {
            let new = a.update < b.update;

            merge(&mut a.typ, &mut b.typ, new);
            merge(&mut a.bounty, &mut b.bounty, new);
            merge(&mut a.severity, &mut b.severity, new);

            a.update = a.update.max(b.update);

            a.subs.append(&mut b.subs);
            a.subs.sort();
            a.subs.dedup_by(Sub::same_bucket);
            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &Filter) -> bool {
        filter
            .scope
            .as_ref()
            .map_or(true, |pat| self.asset.to_lowercase().contains(pat))
            && has(&self.typ, &filter.scope_type)
            && (filter.scope_bounty.is_none() || filter.scope_bounty == self.bounty)
            && (filter.sub.is_none()
                && filter.ip.is_none()
                && filter.port.is_none()
                && filter.service_name.is_none()
                && filter.url.is_none()
                && filter.title.is_none()
                && filter.status_code.is_none()
                && filter.content_type.is_none()
                && filter.content_length.is_none()
                && filter.tech.is_none()
                && filter.tech_version.is_none()
                || self.subs.iter().any(|s| s.matches(filter)))
    }

    pub fn set_name(&mut self, luna: &Luna) {
        for i in 0..self.subs.len() {
            if self.subs[i].asset.is_empty() {
                self.subs[i].set_name(luna);
            }
        }
        for i in 0..self.subs.len() {
            if let Some(scope) = luna.scope(&self.subs[i].asset) {
                self.asset = scope.asset.clone();
                break;
            }
        }
    }
}

impl std::str::FromStr for Scope {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut scope = Self::new();
        scope.asset = s.to_string();
        scope.update = Some(Utc::now());
        Ok(scope)
    }
}
