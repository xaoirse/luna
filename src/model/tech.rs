use super::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Default, Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Tech {
    #[structopt(short, long)]
    pub name: String,

    #[structopt(short, long)]
    pub version: Option<String>,
}
impl Tech {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a.name == b.name {
            a.version = a.version.take().max(b.version.take());
            true
        } else {
            false
        }
    }

    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.name.contains_opt(&filter.tech) && self.version.contains_opt(&filter.tech_version)
    }
}

impl std::str::FromStr for Tech {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tech {
            name: s.to_string(),
            ..Default::default()
        })
    }
}
