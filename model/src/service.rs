use super::*;
use serde::{Deserialize, Serialize};
use clap::Parser;

#[derive(Debug, Serialize, Deserialize, Parser, Clone)]
pub struct Service {
    #[clap(long)]
    pub port: String,

    #[clap(long)]
    pub name: Option<String>,

    #[clap(long)]
    pub protocol: Option<String>,

    #[clap(long)]
    pub banner: Option<String>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Dedup for Service {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        let new = a.update < b.update;

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        if a.port.is_empty() {
            a.port = std::mem::take(&mut b.port);
        }

        merge(&mut a.name, &mut b.name, new);
        merge(&mut a.protocol, &mut b.protocol, new);
        merge(&mut a.banner, &mut b.banner, new);
    }
    fn dedup(&mut self, _term: Arc<AtomicBool>) {}
    fn is_empty(&self) -> bool {
        self.port.is_empty()
    }
}

impl Service {
    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        self.port.contains_opt(&filter.port)
            && self.name.contains_opt(&filter.service_name)
            && self.protocol.contains_opt(&filter.service_protocol)
            && self.banner.contains_opt(&filter.service_banner)
            && (!date
                || (check_date(&self.update, &filter.updated_at)
                    && check_date(&self.start, &filter.started_at)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.port.clone(),
            1 => format!(
                "{} - {}
    Protocol: {}
    Banner: {}
    Update: {}
    Start: {}
    ",
                self.port,
                self.name.as_ref().map_or("", |s| s),
                self.protocol.as_ref().map_or("", |s| s),
                self.banner.as_ref().map_or("", |s| s),
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

impl Default for Service {
    fn default() -> Self {
        Self {
            port: String::new(),
            name: None,
            protocol: None,
            banner: None,
            update: Some(Utc::now()),
            start: Some(Utc::now()),
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

impl PartialEq for Service {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}

impl Eq for Service {}