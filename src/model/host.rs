use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone)]
pub struct Host {
    #[structopt(long)]
    pub ip: String,

    #[structopt(long)]
    pub services: Vec<Service>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,

    #[structopt(skip)]
    pub dedup: bool,
}

impl Dedup for Host {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        if a.ip.is_empty() {
            a.ip = std::mem::take(&mut b.ip);
        }

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.services.append(&mut b.services);
        a.dedup = false;
    }

    fn dedup(&mut self, term: Arc<AtomicBool>) {
        if self.dedup {
            return;
        }
        self.dedup = dedup(&mut self.services, term);
    }
}

impl Host {
    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        filter
            .ip_cidr
            .as_ref()
            .map_or(true, |ip_cidr| match ip_cidr {
                IpCidr::Cidr(cidr) => self
                    .ip
                    .parse::<std::net::IpAddr>()
                    .map_or(false, |ip| cidr.contains(&ip)),
                IpCidr::Ip(ip) => self
                    .ip
                    .parse::<std::net::IpAddr>()
                    .map_or(false, |i| ip == &i),
            })
            && (!date
                || (check_date(&self.update, &filter.updated_at)
                    && check_date(&self.start, &filter.started_at)))
            && (filter.service_is_none()
                || self.services.par_iter().any(|s| s.matches(filter, false)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.ip.to_string(),
            1 => format!(
                "{}
    services: {}
    update: {}
    start: {}
    ",
                self.ip,
                self.services.len(),
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            2 => format!(
                "{}
    services: [{}{}
    update: {}
    start: {}
    ",
                self.ip,
                self.services
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.services.is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
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

impl Default for Host {
    fn default() -> Self {
        Self {
            ip: String::new(),
            services: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
            dedup: false,
        }
    }
}
impl std::str::FromStr for Host {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Host {
            ip: s.parse()?,
            ..Default::default()
        })
    }
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
    }
}

impl Eq for Host {}
