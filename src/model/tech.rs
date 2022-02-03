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

    pub fn matches(&self, filter: &Filter) -> bool {
        filter
            .tech
            .as_ref()
            .map_or(true, |pat| self.name.to_lowercase().contains(pat))
            && (filter.tech_version.is_none() || filter.tech_version == self.version)
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
