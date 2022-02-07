use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Default, Clone, Debug, Serialize, Deserialize, StructOpt, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Sub {
    #[structopt(short, long)]
    pub asset: String,

    #[structopt(short, long,possible_values = &["IP","Domain"] ,case_insensitive= true)]
    pub typ: Option<String>,

    #[structopt(short = "i", long)]
    pub hosts: Vec<Host>,

    #[structopt(short, long)]
    pub urls: Vec<Url>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
}

impl Sub {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if !a.asset.is_empty() && a.asset == b.asset {
            let new = a.update < b.update;

            merge(&mut a.typ, &mut b.typ, new);

            a.update = a.update.max(b.update);

            a.hosts.append(&mut b.hosts);
            a.hosts.par_sort();
            a.hosts.dedup_by(Host::same_bucket);

            a.urls.append(&mut b.urls);
            a.urls.par_sort();
            a.urls.dedup_by(Url::same_bucket);
            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.asset.contains_opt(&filter.sub)
            && self.typ.contains_opt(&filter.sub_typ)
            && (filter.host_is_none() || self.hosts.par_iter().any(|h| h.matches(filter)))
            && (filter.url_is_none() || self.urls.par_iter().any(|u| u.matches(filter)))
    }

    pub fn set_name(&mut self, luna: &Luna) {
        if let Some(url) = self.urls.first() {
            self.asset = url.sub_asset();
        } else if let Some(sub) = self.hosts.par_iter_mut().find_map_any(|h| luna.sub(&h.ip)) {
            self.asset = sub.asset.clone();
        }
    }
}

impl std::str::FromStr for Sub {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Sub {
            asset: s.to_string(),
            update: Some(Utc::now()),
            ..Default::default()
        })
    }
}
