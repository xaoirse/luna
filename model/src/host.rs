use super::*;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Serialize, Deserialize, Parser, Clone)]
pub struct Host {
    #[clap(long)]
    pub ip: IpAddr,

    #[clap(long)]
    pub services: Vec<Service>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Dedup for Host {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.services.append(&mut b.services);
    }

    fn is_empty(&self) -> bool {
        self.ip.is_unspecified()
    }
}

impl Host {
    pub fn clear(&mut self) {
        self.ip = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    }

    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        filter
            .ip_cidr
            .as_ref()
            .map_or(true, |ip_cidr| match ip_cidr {
                IpCidr::Cidr(cidr) => cidr.contains(&self.ip),
                IpCidr::Ip(ip) => &self.ip == ip,
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
            ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            services: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
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

impl Ord for Host {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ip.cmp(&other.ip)
    }
}

impl PartialOrd for Host {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
    }
}

impl Eq for Host {}
