use super::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(
    Default, Debug, Serialize, Deserialize, StructOpt, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Service {
    #[structopt(long)]
    pub port: String,

    #[structopt(long)]
    pub name: Option<String>,

    #[structopt(long)]
    pub protocol: Option<String>,

    #[structopt(short, long)]
    pub banner: Option<String>,
}

impl Service {
    pub fn same_bucket(b: &mut Self, a: &mut Self) -> bool {
        if a.name == b.name && a.port == b.port {
            merge(&mut a.protocol, &mut b.protocol, true);
            merge(&mut a.banner, &mut b.banner, true);

            true
        } else {
            false
        }
    }
    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.port.contains_opt(&filter.port)
            && self.name.contains_opt(&filter.service_name)
            && self.protocol.contains_opt(&filter.service_protocol)
            && self.banner.contains_opt(&filter.service_banner)
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0..=1 => self.port.clone(),
            2..=3 => format!(
                "{} - {},
    protocol: {},
    banner: {}
    ",
                self.port,
                self.name.as_ref().map_or("", |s| s),
                self.protocol.as_ref().map_or("", |s| s),
                self.banner.as_ref().map_or("", |s| s),
            ),
            _ => format!("{:#?}", self),
        }
    }
}

impl std::str::FromStr for Service {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Service {
            port: s.to_string(),
            ..Default::default()
        })
    }
}
