use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::io::Write;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize, PartialEq, Eq)]
pub struct Luna {
    #[structopt(short, long)]
    pub name: String,
    #[structopt(short, long)]
    pub status: String,
    #[structopt(short, long)]
    pub version: String,
    #[structopt(skip)]
    pub counter: i32,
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
        self.programs.par_sort();
        self.programs.dedup_by(Program::same_bucket);
    }

    pub fn find(&self, filter: &FilterRegex) -> Vec<String> {
        match filter.field {
            Fields::Program => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .map(|p| p.stringify(filter.verbose))
                .collect(),
            Fields::Scope => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .map(|s| s.stringify(filter.verbose))
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
                .filter(|u| u.matches(filter))
                .map(|h| h.stringify(filter.verbose))
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

    pub fn program(&self, scope: &str) -> Option<&Program> {
        self.programs
            .par_iter()
            .filter(|p| !p.name.is_empty())
            .find_any(|p| p.scopes.par_iter().any(|s| s.asset == scope))
    }

    pub fn scope(&self, sub: &str) -> Option<&Scope> {
        self.programs
            .par_iter()
            .flat_map(|p| &p.scopes)
            .filter(|s| !s.asset.is_empty())
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

        match std::fs::File::options()
            .write(true)
            .truncate(true)
            .open(path)
        {
            Ok(mut file) => {
                if !Opt::from_args().no_backup {
                    std::fs::copy(path, &format!("{}_{}", Utc::now().to_rfc2822(), path))?;
                }
                Ok(file.write(str.as_bytes())?)
            }
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
                    asset: i.scope.unwrap_or_default(),
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
                    asset: i.scope.unwrap_or_default(),
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
                    asset: i.scope.unwrap_or_default(),
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
                    asset: i.scope.unwrap_or_default(),
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
                    asset: i.scope.unwrap_or_default(),
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
                    asset: i.scope.unwrap_or_default(),
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
        }
    }
}

impl From<Filter> for Luna {
    fn from(mut f: Filter) -> Self {
        let techs = if f.tech_is_none() {
            vec![]
        } else {
            vec![Tech {
                name: f.tech.take().unwrap_or_default(),
                version: f.tech_version.take(),
            }]
        };

        let urls = if f.url_is_none() {
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

        let services = if f.service_is_none() {
            vec![]
        } else {
            vec![Service {
                port: f.port.take().unwrap_or_default(),
                name: f.service_name.take(),
                banner: f.service_banner.take(),
                protocol: f.service_protocol.take(),
            }]
        };

        let hosts = if f.host_is_none() {
            vec![]
        } else {
            vec![Host {
                ip: f.ip.take().unwrap_or_default(),
                services,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let subs = if f.sub_is_none() {
            vec![]
        } else {
            vec![Sub {
                asset: f.sub.take().unwrap_or_default(),
                typ: f.sub_typ.take(),
                hosts,
                urls,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let scopes = if f.scope_is_none() {
            vec![]
        } else {
            vec![Scope {
                asset: f.scope.unwrap_or_default(),
                typ: f.scope_type,
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
