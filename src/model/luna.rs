use super::*;
use crate::model::url::Url;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
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

    #[structopt(skip)]
    dedup: bool,
}

impl Luna {
    pub fn append(&mut self, mut other: Self) {
        self.counter += other.counter;
        self.programs.append(&mut other.programs);
        self.dedup = false;

        self.update = self.update.max(other.update);
        self.start = self.start.min(other.start);
    }

    pub fn dedup(&mut self, term: Arc<AtomicBool>) {
        if self.dedup {
            return;
        }
        dedup(&mut self.programs, term);
        self.dedup = true;
    }

    pub fn find(&self, field: Fields, filter: &FilterRegex) -> Vec<String> {
        match field {
            Fields::Luna => vec![self.stringify(filter.verbose)],
            Fields::Program => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .map(|p| p.stringify(filter.verbose))
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
                .filter(|h| h.matches(filter))
                .map(|h| h.stringify(filter.verbose))
                .collect(),
            Fields::Service => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.hosts)
                .filter(|h| h.matches(filter))
                .flat_map(|h| &h.services)
                .filter(|s| s.matches(filter))
                .map(|s| s.stringify(filter.verbose))
                .collect(),
            Fields::Tech => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.urls)
                .filter(|u| u.matches(filter))
                .flat_map(|u| &u.techs)
                .filter(|t| t.matches(filter))
                .map(|t| t.stringify(filter.verbose))
                .collect(),
            Fields::Tag => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter))
                .flat_map(|s| &s.urls)
                .filter(|u| u.matches(filter))
                .flat_map(|u| &u.tags)
                .filter(|t| t.matches(filter))
                .map(|t| t.stringify(filter.verbose))
                .collect(),
            Fields::Keyword => todo!(),
            Fields::None => vec!["".to_string()],
        }
    }

    pub fn programs(&mut self, filter: &FilterRegex) -> Vec<&mut Program> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter))
            .collect()
    }

    pub fn scopes(&mut self, filter: &FilterRegex) -> Vec<&mut Scope> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter))
            .collect()
    }
    pub fn subs(&mut self, filter: &FilterRegex) -> Vec<&mut Sub> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter))
            .flat_map(|s| &mut s.subs)
            .filter(|s| s.matches(filter))
            .collect()
    }

    pub fn urls(&mut self, filter: &FilterRegex) -> Vec<&mut Url> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter))
            .flat_map(|s| &mut s.subs)
            .filter(|s| s.matches(filter))
            .flat_map(|s| &mut s.urls)
            .filter(|u| u.matches(filter))
            .collect()
    }

    pub fn hosts(&mut self, filter: &FilterRegex) -> Vec<&mut Host> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter))
            .flat_map(|s| &mut s.subs)
            .filter(|s| s.matches(filter))
            .flat_map(|s| &mut s.hosts)
            .filter(|h| h.matches(filter))
            .collect()
    }

    fn save_as(&self, path: &str) -> Result<usize, Errors> {
        let str = serde_json::to_string(&self)?;

        if !Opt::from_args().no_backup && std::path::Path::new(path).exists() {
            let copy_path = match path.rsplit_once('.') {
                Some((a, b)) => format!("{}_{}.{}", a, chrono::Local::now().to_rfc2822(), b),
                None => format!("{}_{}", path, Utc::now().to_rfc2822()),
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

    pub fn save(&self) {
        let opt = Opt::from_args();
        let output = opt.output.as_ref().unwrap_or(&opt.input);

        if let Err(err) = self.save_as(output) {
            error!("Error while saving: {}", err);
        } else {
            info!("Saved in \"{}\" successfully.", output);
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Errors> {
        let file = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file)?)
    }
    pub fn from_args() -> Luna {
        let opt = Opt::from_args();

        match Luna::from_file(&opt.input) {
            Ok(luna) => {
                info!("Luna loaded successfully.");
                luna
            }
            Err(err) => {
                if err.to_string() == "No such file or directory (os error 2)" {
                    warn!("Can't load Luna from file! New filw will be generated.")
                } else {
                    error!("Can't load Luna from file!: {}", err);
                }
                Luna::default()
            }
        }
    }

    pub fn remove(&mut self, field: Fields, filter: &FilterRegex) -> bool {
        let len = self.find(field, filter).len();
        if len == 1 {
            match field {
                Fields::Program => self.programs.retain(|p| !p.matches(filter)),
                Fields::Domain => self
                    .programs(filter)
                    .first_mut()
                    .unwrap()
                    .scopes
                    .retain(|p| !p.matches(filter)),
                Fields::Cidr => self
                    .programs(filter)
                    .first_mut()
                    .unwrap()
                    .scopes
                    .retain(|p| !p.matches(filter)),
                Fields::Sub => self
                    .scopes(filter)
                    .first_mut()
                    .unwrap()
                    .subs
                    .retain(|s| !s.matches(filter)),
                Fields::Url => self
                    .subs(filter)
                    .first_mut()
                    .unwrap()
                    .urls
                    .retain(|s| !s.matches(filter)),
                Fields::IP => self
                    .subs(filter)
                    .first_mut()
                    .unwrap()
                    .hosts
                    .retain(|h| !h.matches(filter)),
                Fields::Tag => self
                    .urls(filter)
                    .first_mut()
                    .unwrap()
                    .tags
                    .retain(|t| !t.matches(filter)),
                Fields::Tech => self
                    .urls(filter)
                    .first_mut()
                    .unwrap()
                    .techs
                    .retain(|t| !t.matches(filter)),
                Fields::Service => self
                    .hosts(filter)
                    .first_mut()
                    .unwrap()
                    .services
                    .retain(|t| !t.matches(filter)),
                Fields::Keyword => todo!(),
                Fields::None => error!("what are you trying to delete?"),
                Fields::Luna => error!("Stupid! Do you want to delete Luna?"),
            }

            return true;
        } else if len == 0 {
            warn!("No items found!")
        } else {
            error!("For security reasons you can't delete multi fields at once!")
        }
        false
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
    Services: {}
    Techs: {}
    Tags: {}
    Update: {}
    Start: {}
    ",
                self.name,
                self.version,
                self.status,
                self.counter,
                self.programs.iter().filter(|p| !p.name.is_empty()).count(),
                self.find(Fields::Domain, &FilterRegex::default()).len(),
                self.find(Fields::Cidr, &FilterRegex::default()).len(),
                self.find(Fields::Sub, &FilterRegex::default()).len(),
                self.find(Fields::IP, &FilterRegex::default()).len(),
                self.find(Fields::Url, &FilterRegex::default()).len(),
                self.find(Fields::Service, &FilterRegex::default()).len(),
                self.find(Fields::Tech, &FilterRegex::default()).len(),
                self.find(Fields::Tag, &FilterRegex::default()).len(),
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
    Services: {}
    Techs: {}
    Tags: {}
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
                if self.programs.iter().filter(|p| !p.name.is_empty()).count() == 0 {
                    "]"
                } else {
                    "\n    ]"
                },
                self.find(Fields::Domain, &FilterRegex::default()).len(),
                self.find(Fields::Cidr, &FilterRegex::default()).len(),
                self.find(Fields::Sub, &FilterRegex::default()).len(),
                self.find(Fields::IP, &FilterRegex::default()).len(),
                self.find(Fields::Url, &FilterRegex::default()).len(),
                self.find(Fields::Service, &FilterRegex::default()).len(),
                self.find(Fields::Tech, &FilterRegex::default()).len(),
                self.find(Fields::Tag, &FilterRegex::default()).len(),
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
            dedup: false,
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
                        asset: i
                            .sub
                            .unwrap_or_else(|| i.url.sub_asset().unwrap_or_default()),
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
                        asset: i.sub.unwrap_or_else(|| {
                            i.url
                                .split('/')
                                .nth(2)
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        }),
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

impl From<InsertTag> for Luna {
    fn from(i: InsertTag) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i.sub.unwrap_or_else(|| {
                            i.url
                                .split('/')
                                .nth(2)
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        }),
                        urls: vec![Url {
                            url: i.url,
                            tags: vec![Tag { ..i.tag }],
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
            Insert::Tag(i) => i.into(),
            Insert::Service(i) => i.into(),
        }
    }
}

impl From<Filter> for Luna {
    fn from(mut f: Filter) -> Self {
        let tech_is_none = f.tech_is_none();
        let tag_is_none = f.tag_is_none();
        let url_is_none = f.url_is_none();
        let service_is_none = f.service_is_none();
        let host_is_none = f.host_is_none();
        let sub_is_none = f.sub_is_none();
        let scope_is_none = f.scope_is_none();

        let techs = if tech_is_none {
            vec![]
        } else if let Some(tech) = f.tech {
            if tech.split(',').count() > 1 {
                tech.split(',')
                    .map(|t| Tech::from_str(t).unwrap())
                    .collect()
            } else {
                vec![Tech {
                    name: tech,
                    version: f.tech_version,
                    update: Some(Utc::now()),
                    start: Some(Utc::now()),
                }]
            }
        } else {
            vec![Tech {
                name: f.tech.unwrap_or_default(),
                version: f.tech_version,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let tags = if tag_is_none {
            vec![]
        } else {
            vec![Tag {
                name: f.tag.unwrap_or_default(),
                severity: f.tag_severity,
                values: f
                    .tag_value
                    .take()
                    .unwrap_or_default()
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                ..Default::default()
            }]
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
                tags,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
                dedup: false,
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
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let hosts = if host_is_none {
            vec![]
        } else if let Some(ip) = f.ip.take() {
            if !ip.contains(',') {
                vec![Host {
                    ip,
                    services,
                    update: Some(Utc::now()),
                    start: Some(Utc::now()),
                    dedup: false,
                }]
            } else {
                ip.split(',').map(|s| Host::from_str(s).unwrap()).collect()
            }
        } else {
            vec![Host {
                ip: String::new(),
                services,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
                dedup: false,
            }]
        };

        let subs = if sub_is_none {
            vec![]
        } else {
            vec![Sub {
                asset: f.sub.take().unwrap_or_else(|| {
                    urls.first()
                        .map(|u| {
                            u.url
                                .split('/')
                                .nth(2)
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default()
                }),
                typ: f.sub_type.take(),
                hosts,
                urls,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
                dedup: false,
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
                dedup: false,
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
                dedup: false,
            }],
            ..Default::default()
        }
    }
}
