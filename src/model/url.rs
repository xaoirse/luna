use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Debug, Default, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Url {
    #[structopt(short, long)]
    pub url: String,

    #[structopt(long)]
    pub title: Option<String>,

    #[structopt(long)]
    pub status_code: Option<String>,

    #[structopt(long, short)]
    pub response: Option<String>,

    #[structopt(long)]
    pub techs: Vec<Tech>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
}

impl Url {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a.url == b.url {
            let new = a.update < b.update;

            merge(&mut a.title, &mut b.title, new);
            merge(&mut a.status_code, &mut b.status_code, new);
            merge(&mut a.response, &mut b.response, new);

            a.update = a.update.max(b.update);

            a.techs.append(&mut b.techs);
            a.techs.par_sort();
            a.techs.dedup_by(Tech::same_bucket);

            true
        } else {
            false
        }
    }
    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.url.contains_opt(&filter.url)
            && self.title.contains_opt(&filter.title)
            && self.response.contains_opt(&filter.response)
            && self.status_code.contains_opt(&filter.status_code)
            && (filter.tech_is_none() || self.techs.par_iter().any(|t| t.matches(filter)))
    }

    pub fn sub_asset(&self) -> String {
        todo!()
    }
}

impl std::str::FromStr for Url {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Url {
            url: s.to_string(),
            update: Some(Utc::now()),
            ..Default::default()
        })
    }
}

// impl<'t> From<regex::Captures<'t>> for Url {
//     fn from(cap: regex::Captures<'t>) -> Self {
//         Self {
//             url: cap
//                 .get(0)
//                 .map_or("".to_string(), |m| m.as_str().to_string()),
//             title: None,
//             status_code: None,
//             response: vec![],
//             techs: vec![],
//             update: Some(Utc::now()),
//         }
//     }
// }
