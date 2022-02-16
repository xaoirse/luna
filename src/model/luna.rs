use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::io::Write;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct Luna {
    #[structopt(short, long)]
    pub name: String,
    #[structopt(short, long)]
    pub status: String,
    #[structopt(short, long)]
    pub version: String,
    #[structopt(skip)]
    pub counter: i64,
    #[structopt(short, long)]
    pub programs: Vec<Program>,
    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Luna {
    pub fn append(&mut self, mut other: Self) {
        // Append
        self.counter += other.counter;
        self.programs.append(&mut other.programs);

        self.update = self.update.max(other.update);
        self.start = self.start.min(other.start);
    }
    pub fn merge(&mut self) {
        // Fill nones
        let luna_copy = self.clone();
        self.programs
            .par_iter_mut()
            .for_each(|p| p.set_name(&luna_copy));

        // Merge
        self.programs
            .par_sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.programs.dedup_by(Program::same_bucket);
    }

    pub fn find(&self, filter: &FilterRegex) -> Vec<String> {
        match filter.field {
            Fields::Program => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .map(|p| p.stringify(filter.verbose))
                .filter(|s| !s.is_empty())
                .collect(),
            Fields::Domain => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .filter_map(|s| match &s.asset {
                    ScopeType::Domain(_) => Some(s.stringify(filter.verbose)),
                    _ => None,
                })
                .filter(|s| !s.is_empty())
                .collect(),
            Fields::Cidr => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .filter_map(|s| match &s.asset {
                    ScopeType::Cidr(_) => Some(s.stringify(filter.verbose)),
                    _ => None,
                })
                .filter(|s| !s.is_empty())
                .collect(),
            Fields::Sub => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter))
                .map(|s| s.stringify(filter.verbose))
                .filter(|s| !s.is_empty())
                .collect(),
            Fields::Url => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.urls)
                .filter(|u| u.matches(filter))
                .map(|u| u.stringify(filter.verbose))
                .filter(|s| !s.is_empty())
                .collect(),
            Fields::IP => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.hosts)
                .filter(|h| h.matches(filter))
                .map(|h| h.stringify(filter.verbose))
                .filter(|s| !s.is_empty())
                .collect(),
            Fields::None => vec!["".to_string()],
            Fields::Service => todo!(),
            Fields::Tech => todo!(),
            Fields::Keyword => todo!(),
        }
    }
    pub fn find_all(&self, field: Fields) -> Vec<String> {
        self.find(&FilterRegex {
            field,
            ..Default::default()
        })
    }

    pub fn program(&self, scope: &ScopeType) -> Option<&Program> {
        self.programs
            .par_iter()
            .filter(|p| !p.name.is_empty())
            .find_any(|p| p.scopes.par_iter().any(|s| &s.asset == scope))
    }

    pub fn scope(&self, sub: &str) -> Option<&Scope> {
        self.programs
            .par_iter()
            .flat_map(|p| &p.scopes)
            .filter(|s| s.asset != ScopeType::Empty)
            .find_any(|s| s.subs.par_iter().any(|s| s.asset == sub))
    }
    pub fn sub(&self, ip: &str) -> Option<&Sub> {
        self.programs
            .par_iter()
            .flat_map(|p| &p.scopes)
            .flat_map(|s| &s.subs)
            .filter(|s| !s.asset.is_empty())
            .find_any(|s| s.hosts.par_iter().any(|h| h.ip == ip))
    }

    pub fn save(&self, path: &str) -> Result<usize, Errors> {
        let str = serde_json::to_string(&self)?;

        if !Opt::from_args().no_backup && std::path::Path::new(path).exists() {
            let copy_path = match path.rsplit_once('.') {
                Some((a, b)) => format!("{}_{}.{}", a, chrono::Local::now().to_rfc2822(), b),
                None => format!("{}{}", path, Utc::now().to_rfc2822()),
            };
            std::fs::copy(path, copy_path)?;
        }
        match std::fs::File::options()
            .write(true)
            .truncate(true)
            .open(path)
        {
            Ok(mut file) => Ok(file.write(str.as_bytes())?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Ok(std::fs::File::create(path)?.write(str.as_bytes())?)
            }
            Err(err) => Err(Box::new(err)),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Errors> {
        let file = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file)?)
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!("{} {} ", self.name, self.version),
            2 => format!(
                "{}  {}
    Status: {}
    Counter: {}
    Programs: {}
    Domains: {}
    CIDRs: {}
    Subs: {}
    IPs: {}
    URLs: {}
    Update: {}
    Start: {}
    ",
                self.name,
                self.version,
                self.status,
                self.counter,
                self.programs.iter().filter(|p| !p.name.is_empty()).count(),
                self.find_all(Fields::Domain).len(),
                self.find_all(Fields::Cidr).len(),
                self.find_all(Fields::Sub).len(),
                self.find_all(Fields::IP).len(),
                self.find_all(Fields::Url).len(),
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            3 => format!(
                "{}  {}
    Status: {}
    Counter: {}
    Programs: [{}{}
    Domains: {}
    CIDRs: {}
    Subs: {}
    IPs: {}
    URLs: {}
    Update: {}
    Start: {}
    ",
                self.name,
                self.version,
                self.status,
                self.counter,
                self.programs
                    .iter()
                    .filter(|p| !p.name.is_empty())
                    .map(|s| format!("\n        {}", s.stringify(1)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.programs.is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
                self.find_all(Fields::Domain).len(),
                self.find_all(Fields::Cidr).len(),
                self.find_all(Fields::Sub).len(),
                self.find_all(Fields::IP).len(),
                self.find_all(Fields::Url).len(),
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

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Default for Luna {
    fn default() -> Self {
        Self {
            name: "Luna".to_string(),
            version: VERSION.to_string(),
            counter: 1,
            programs: vec![],
            status: "The moon rider has arrived.".to_string(),
            update: Some(Utc::now()),
            start: Some(Utc::now()),
        }
    }
}

impl From<InsertProgram> for Luna {
    fn from(i: InsertProgram) -> Self {
        Luna {
            programs: vec![Program {
                start: Some(Utc::now()),
                update: Some(Utc::now()),
                ..i.program
            }],
            ..Default::default()
        }
    }
}
impl From<InsertScope> for Luna {
    fn from(i: InsertScope) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    update: Some(Utc::now()),
                    start: Some(Utc::now()),
                    ..i.scope
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}
impl From<InsertScopes> for Luna {
    fn from(i: InsertScopes) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: i.scopes,
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertSub> for Luna {
    fn from(i: InsertSub) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        update: Some(Utc::now()),
                        start: Some(Utc::now()),
                        ..i.sub
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertSubs> for Luna {
    fn from(i: InsertSubs) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: i.subs,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertUrl> for Luna {
    fn from(i: InsertUrl) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i.sub.unwrap_or_default(),
                        urls: vec![Url {
                            update: Some(Utc::now()),
                            start: Some(Utc::now()),
                            ..i.url
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertUrls> for Luna {
    fn from(i: InsertUrls) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i.sub.unwrap_or_default(),
                        urls: i.urls,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertHost> for Luna {
    fn from(i: InsertHost) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i.sub.unwrap_or_default(),
                        hosts: vec![Host {
                            update: Some(Utc::now()),
                            start: Some(Utc::now()),
                            ..i.host
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertHosts> for Luna {
    fn from(i: InsertHosts) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i.sub.unwrap_or_default(),
                        hosts: i.hosts,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertTech> for Luna {
    fn from(i: InsertTech) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i.sub.unwrap_or_default(),
                        urls: vec![Url {
                            url: i.url,
                            techs: vec![Tech { ..i.tech }],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<InsertService> for Luna {
    fn from(i: InsertService) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i.sub.unwrap_or_default(),
                        hosts: vec![Host {
                            ip: i.host,
                            services: vec![Service { ..i.service }],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<Insert> for Luna {
    fn from(i: Insert) -> Self {
        match i {
            Insert::Program(i) => i.into(),
            Insert::Scope(i) => i.into(),
            Insert::Scopes(i) => i.into(),
            Insert::Sub(i) => i.into(),
            Insert::Subs(i) => i.into(),
            Insert::Url(i) => i.into(),
            Insert::Urls(i) => i.into(),
            Insert::Host(i) => i.into(),
            Insert::Hosts(i) => i.into(),
            Insert::Tech(i) => i.into(),
            Insert::Service(i) => i.into(),
        }
    }
}

impl From<Filter> for Luna {
    fn from(mut f: Filter) -> Self {
        let tech_is_none = f.tech_is_none();
        let url_is_none = f.url_is_none();
        let service_is_none = f.service_is_none();
        let host_is_none = f.host_is_none();
        let sub_is_none = f.sub_is_none();
        let scope_is_none = f.scope_is_none();

        let techs = if tech_is_none {
            vec![]
        } else {
            f.tech
                .take()
                .unwrap_or_default()
                .split(',')
                .map(|t| Tech::from_str(t).unwrap())
                .collect()
        };

        let urls = if url_is_none {
            vec![]
        } else {
            vec![Url {
                url: f.url.take().unwrap_or_default(),
                title: f.title.take(),
                status_code: f.status_code.take(),
                response: f.response.take(),
                techs,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let services = if service_is_none {
            vec![]
        } else {
            vec![Service {
                port: f.port.take().unwrap_or_default(),
                name: f.service_name.take(),
                banner: f.service_banner.take(),
                protocol: f.service_protocol.take(),
            }]
        };

        let hosts = if host_is_none {
            vec![]
        } else {
            vec![Host {
                ip: f.ip.take().unwrap_or_default(),
                services,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let subs = if sub_is_none {
            vec![]
        } else {
            vec![Sub {
                asset: f.sub.take().unwrap_or_default(),
                typ: f.sub_type.take(),
                hosts,
                urls,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let scopes = if scope_is_none {
            vec![]
        } else {
            vec![Scope {
                asset: ScopeType::from_str(&f.scope.unwrap_or_default()).unwrap(),
                severity: f.scope_severity,
                bounty: f.scope_bounty,
                subs,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        Self {
            programs: vec![Program {
                name: f.program.unwrap_or_default(),
                platform: f.program_platform,
                typ: f.program_type,
                handle: f.program_handle,
                url: f.program_url,
                bounty: f.program_bounty,
                icon: f.program_icon,
                state: f.program_state,
                scopes,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }],
            ..Default::default()
        }
    }
}
