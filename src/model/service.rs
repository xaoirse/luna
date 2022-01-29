use super::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Default, Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Service {
    #[structopt(short, long)]
    pub port: String,

    #[structopt(long)]
    pub name: Option<String>,

    #[structopt(long)]
    pub protocol: Option<String>,

    #[structopt(short, long)]
    pub banner: Option<String>,
}

impl Service {
    fn new() -> Self {
        Default::default()
    }

    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a.name == b.name && a.port == b.port {
            merge(&mut a.protocol, &mut b.protocol, true);
            merge(&mut a.banner, &mut b.banner, true);

            true
        } else {
            false
        }
    }
    pub fn matches(&self, filter: &Filter) -> bool {
        filter
            .port
            .as_ref()
            .map_or(true, |pat| self.port.to_lowercase().contains(pat))
            && has(&self.name, &filter.service_name)
    }
}

impl std::str::FromStr for Service {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut service = Self::new();
        service.port = s.to_string();
        Ok(service)
    }
}
