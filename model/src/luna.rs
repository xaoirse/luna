use super::*;
use ::url as urlib;
use chrono::{DateTime, Utc};
use clap::Parser;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct Luna {
    #[clap(short, long)]
    pub name: String,

    #[clap(long)]
    pub status: String,

    #[clap(long)]
    pub version: String,

    #[clap(skip)]
    pub counter: i64,

    #[clap(short, long)]
    pub programs: Vec<Program>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
    pub start: Option<DateTime<Utc>>,
}

impl Luna {
    pub fn append(&mut self, other: Self) {
        self.counter += other.counter;

        self.update = self.update.max(other.update);
        self.start = self.start.min(other.start);

        let filter = &FilterRegex::default();

        other.programs.into_iter().for_each(|p| {
            if p.name.is_empty() {
                p.scopes.into_iter().for_each(|s| {
                    if s.asset == ScopeType::Empty {
                        s.subs.into_iter().for_each(|s| {
                            if s.asset.is_empty() {
                                s.urls.into_iter().for_each(|u| {
                                    let urls = self.urls(filter);
                                    for su in urls {
                                        if su == &u {
                                            Url::same(u, su);
                                            return;
                                        }
                                    }
                                    self.programs.push(Program {
                                        scopes: vec![Scope {
                                            subs: vec![Sub {
                                                urls: vec![u],
                                                ..Default::default()
                                            }],
                                            ..Default::default()
                                        }],
                                        ..Default::default()
                                    });
                                });
                                s.hosts.into_iter().for_each(|h| {
                                    let hosts = self.hosts(filter);
                                    for sh in hosts {
                                        if sh == &h {
                                            Host::same(h, sh);
                                            return;
                                        }
                                    }
                                    self.programs.push(Program {
                                        scopes: vec![Scope {
                                            subs: vec![Sub {
                                                hosts: vec![h],
                                                ..Default::default()
                                            }],
                                            ..Default::default()
                                        }],
                                        ..Default::default()
                                    });
                                });
                            } else {
                                let subs = self.subs(filter);
                                for ss in subs {
                                    if ss == &s {
                                        Sub::same(s, ss);
                                        return;
                                    }
                                }

                                if let Some(ss) = self
                                    .programs
                                    .iter_mut()
                                    .flat_map(|p| &mut p.scopes)
                                    .find(|ss| match &ss.asset {
                                        ScopeType::Domain(d) => s.asset.ends_with(&d.to_string()),
                                        _ => false,
                                    })
                                {
                                    if let Some(a) = ss.subs.iter_mut().find(|a| &&s == a) {
                                        Sub::same(s, a);
                                    } else {
                                        ss.subs.push(s);
                                    }
                                } else {
                                    self.programs.push(Program {
                                        scopes: vec![Scope {
                                            subs: vec![s],
                                            ..Default::default()
                                        }],
                                        ..Default::default()
                                    });
                                }
                            }
                        })
                    } else {
                        let scopes = self.scopes(filter);
                        for ss in scopes {
                            if ss == &s {
                                Scope::same(s, ss);
                                return;
                            }
                        }
                        self.programs.push(Program {
                            scopes: vec![s],
                            ..Default::default()
                        });
                    }
                })
            } else {
                for i in 0..self.programs.len() {
                    if self.programs[i] == p {
                        Program::same(p, &mut self.programs[i]);
                        return;
                    }
                }
                self.programs.push(p);
            }
        })
    }

    pub fn dedup(&mut self, term: Arc<AtomicBool>) {
        dedup(&mut self.programs, term);
        self.programs
            .par_iter_mut()
            .for_each(|p| p.scopes.retain(|s| s.asset != ScopeType::Empty));
        self.programs.retain(|p| !p.is_empty());
    }

    pub fn find(&self, field: Fields, filter: &FilterRegex, verbose: u8) -> Vec<String> {
        match field {
            Fields::Luna => vec![self.stringify(verbose)],
            Fields::Program => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, true))
                .map(|p| p.stringify(verbose))
                .collect(),
            Fields::Domain => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, false))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter, true))
                .filter_map(|s| match &s.asset {
                    ScopeType::Domain(_) => Some(s.stringify(verbose)),
                    _ => None,
                })
                .collect(),
            Fields::Cidr => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, false))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter, true))
                .filter_map(|s| match &s.asset {
                    ScopeType::Cidr(d) => {
                        if verbose == 3 {
                            Some(
                                d.iter()
                                    .map(|c| c.address().to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n"),
                            )
                        } else {
                            Some(s.stringify(verbose))
                        }
                    }
                    _ => None,
                })
                .collect(),
            Fields::Sub => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, false))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter, true))
                .map(|s| s.stringify(verbose))
                .collect(),
            Fields::Url => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, false))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.urls)
                .filter(|u| u.matches(filter, true))
                .map(|u| u.stringify(verbose))
                .collect(),
            Fields::IP => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, false))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.hosts)
                .filter(|h| h.matches(filter, true))
                .map(|h| h.stringify(verbose))
                .collect(),
            Fields::Service => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, false))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.hosts)
                .filter(|h| h.matches(filter, false))
                .flat_map(|h| &h.services)
                .filter(|s| s.matches(filter, true))
                .map(|s| s.stringify(verbose))
                .collect(),
            Fields::Tag => self
                .programs
                .par_iter()
                .filter(|p| p.matches(filter, false))
                .flat_map(|p| &p.scopes)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.subs)
                .filter(|s| s.matches(filter, false))
                .flat_map(|s| &s.urls)
                .filter(|u| u.matches(filter, false))
                .flat_map(|u| &u.tags)
                .filter(|t| t.matches(filter, true))
                .map(|t| t.stringify(verbose))
                .collect(),
            Fields::Keyword => todo!(),
            Fields::None => vec!["".to_string()],
        }
    }

    pub fn programs(&mut self, filter: &FilterRegex) -> Vec<&mut Program> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter, true))
            .collect()
    }

    pub fn scopes(&mut self, filter: &FilterRegex) -> Vec<&mut Scope> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter, false))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter, true))
            .collect()
    }
    pub fn subs(&mut self, filter: &FilterRegex) -> Vec<&mut Sub> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter, false))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter, false))
            .flat_map(|s| &mut s.subs)
            .filter(|s| s.matches(filter, true))
            .collect()
    }

    pub fn urls(&mut self, filter: &FilterRegex) -> Vec<&mut Url> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter, false))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter, false))
            .flat_map(|s| &mut s.subs)
            .filter(|s| s.matches(filter, false))
            .flat_map(|s| &mut s.urls)
            .filter(|u| u.matches(filter, true))
            .collect()
    }

    pub fn hosts(&mut self, filter: &FilterRegex) -> Vec<&mut Host> {
        self.programs
            .par_iter_mut()
            .filter(|p| p.matches(filter, false))
            .flat_map(|p| &mut p.scopes)
            .filter(|s| s.matches(filter, false))
            .flat_map(|s| &mut s.subs)
            .filter(|s| s.matches(filter, false))
            .flat_map(|s| &mut s.hosts)
            .filter(|h| h.matches(filter, true))
            .collect()
    }

    pub fn save_as(&self, path: &str) -> Result<usize, Errors> {
        let str = serde_json::to_string(&self)?;

        if !Opt::parse().no_backup && std::path::Path::new(path).exists() {
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
        let opt = Opt::parse();
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
    pub fn parse() -> Luna {
        let opt = Opt::parse();

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
        let len = self.find(field, filter, 0).len();
        if len == 1 {
            match field {
                Fields::Program => self.programs.retain(|p| !p.matches(filter, true)),
                Fields::Domain => self
                    .programs(filter)
                    .first_mut()
                    .unwrap()
                    .scopes
                    .retain(|p| !p.matches(filter, true)),
                Fields::Cidr => self
                    .programs(filter)
                    .first_mut()
                    .unwrap()
                    .scopes
                    .retain(|p| !p.matches(filter, true)),
                Fields::Sub => self
                    .scopes(filter)
                    .first_mut()
                    .unwrap()
                    .subs
                    .retain(|s| !s.matches(filter, true)),
                Fields::Url => self
                    .subs(filter)
                    .first_mut()
                    .unwrap()
                    .urls
                    .retain(|s| !s.matches(filter, true)),
                Fields::IP => self
                    .subs(filter)
                    .first_mut()
                    .unwrap()
                    .hosts
                    .retain(|h| !h.matches(filter, true)),
                Fields::Tag => self
                    .urls(filter)
                    .first_mut()
                    .unwrap()
                    .tags
                    .retain(|t| !t.matches(filter, true)),
                Fields::Service => self
                    .hosts(filter)
                    .first_mut()
                    .unwrap()
                    .services
                    .retain(|t| !t.matches(filter, true)),
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
            1 => format!("{} {}", self.name, self.version),
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
    Tags: {}
    Update: {}
    Start: {}
    ",
                self.name,
                self.version,
                self.status,
                self.counter,
                self.programs.iter().filter(|p| !p.name.is_empty()).count(),
                self.find(Fields::Domain, &FilterRegex::default(), 0).len(),
                self.find(Fields::Cidr, &FilterRegex::default(), 0).len(),
                self.find(Fields::Sub, &FilterRegex::default(), 0).len(),
                self.find(Fields::IP, &FilterRegex::default(), 0).len(),
                self.find(Fields::Url, &FilterRegex::default(), 0).len(),
                self.find(Fields::Service, &FilterRegex::default(), 0).len(),
                self.find(Fields::Tag, &FilterRegex::default(), 0).len(),
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
                self.find(Fields::Domain, &FilterRegex::default(), 0).len(),
                self.find(Fields::Cidr, &FilterRegex::default(), 0).len(),
                self.find(Fields::Sub, &FilterRegex::default(), 0).len(),
                self.find(Fields::IP, &FilterRegex::default(), 0).len(),
                self.find(Fields::Url, &FilterRegex::default(), 0).len(),
                self.find(Fields::Service, &FilterRegex::default(), 0).len(),
                self.find(Fields::Tag, &FilterRegex::default(), 0).len(),
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
                        asset: i.sub.unwrap_or_else(|| {
                            i.url.url.host_str().unwrap_or_default().to_string()
                        }),
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

impl From<InsertTag> for Luna {
    fn from(i: InsertTag) -> Self {
        Luna {
            programs: vec![Program {
                name: i.program.unwrap_or_default(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str(&i.scope.unwrap_or_default()).unwrap(),
                    subs: vec![Sub {
                        asset: i
                            .sub
                            .unwrap_or_else(|| i.url.host_str().unwrap_or_default().to_string()),
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
            Insert::Tag(i) => i.into(),
            Insert::Service(i) => i.into(),
        }
    }
}

impl From<Filter> for Luna {
    #[allow(clippy::or_fun_call)]
    fn from(mut f: Filter) -> Self {
        let tag_is_none = f.tag_is_none();
        let url_is_none = f.url_is_none();
        let service_is_none = f.service_is_none();
        let host_is_none = f.host_is_none();
        let sub_is_none = f.sub_is_none();
        let scope_is_none = f.scope_is_none();

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
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let urls = if url_is_none {
            vec![]
        } else if let Ok(url) = urlib::Url::parse(&f.url.unwrap_or_default()) {
            vec![Url {
                url,
                title: f.title.take(),
                status_code: f.status_code.take(),
                response: f.response.take(),
                tags,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        } else {
            vec![]
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
            if ip.contains(',') {
                ip.split(',')
                    .filter_map(|s| Host::from_str(s).ok())
                    .collect()
            } else if let Ok(ip) = std::net::IpAddr::from_str(&ip) {
                vec![Host {
                    ip,
                    services,
                    update: Some(Utc::now()),
                    start: Some(Utc::now()),
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let subs = if sub_is_none {
            vec![]
        } else {
            let asset = match urls.first() {
                Some(url) => url
                    .url
                    .host_str()
                    .unwrap_or(f.sub.take().unwrap_or_default().as_str())
                    .to_string(),
                None => f.sub.take().unwrap_or_default(),
            };

            vec![Sub {
                asset,
                typ: f.sub_type.take(),
                hosts,
                urls,
                update: Some(Utc::now()),
                start: Some(Utc::now()),
            }]
        };

        let scopes = if scope_is_none {
            vec![]
        } else if let (Some(scope), Some(sub)) = (f.scope.as_ref(), subs.first()) {
            if sub.asset.ends_with(scope) {
                if let Ok(scope) = ScopeType::from_str(&f.scope.unwrap_or_default()) {
                    vec![Scope {
                        asset: scope,
                        severity: f.scope_severity,
                        bounty: f.scope_bounty,
                        subs,
                        update: Some(Utc::now()),
                        start: Some(Utc::now()),
                    }]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
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

mod test {

    #[test]
    pub fn test_run() {
        use super::*;

        const N: usize = 10000;
        const M: usize = 10000; // 10 contains 1

        let mut luna = Luna::default();

        for i in 0..N {
            let l = Luna {
                programs: vec![Program {
                    name: "".to_string(),
                    scopes: vec![Scope {
                        asset: ScopeType::from_str("luna.test").unwrap(),
                        subs: vec![Sub {
                            asset: "luna.test".to_string(),
                            urls: vec![Url::from_str(&format!("https://luna.test?{}", i)).unwrap()],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            };
            luna.append(l);
        }
        let term = Arc::new(AtomicBool::new(false));
        luna.dedup(term);

        assert_eq!(luna.programs.len(), 1);
        assert_eq!(luna.programs.first().unwrap().scopes.len(), 1);
        assert_eq!(
            luna.programs
                .first()
                .unwrap()
                .scopes
                .first()
                .unwrap()
                .subs
                .len(),
            1
        );
        assert_eq!(
            luna.programs
                .first()
                .unwrap()
                .scopes
                .first()
                .unwrap()
                .subs
                .first()
                .unwrap()
                .urls
                .len(),
            M
        );
    }

    #[test]
    fn dedup() {
        use super::*;

        let mut a = Luna {
            programs: vec![Program {
                name: "A".to_string(),
                scopes: vec![Scope {
                    asset: ScopeType::Empty,
                    subs: vec![Sub {
                        asset: "luna.test".to_string(),
                        urls: vec![Url::from_str("https://luna.test?1").unwrap()],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        let b = Luna {
            programs: vec![Program {
                name: "B".to_string(),
                scopes: vec![Scope {
                    asset: ScopeType::Empty,
                    subs: vec![Sub {
                        asset: "luna.test".to_string(),
                        urls: vec![Url::from_str("https://luna.test?1").unwrap()],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        let c = Luna {
            programs: vec![Program {
                name: "C".to_string(),
                scopes: vec![Scope {
                    asset: ScopeType::Empty,
                    subs: vec![Sub {
                        asset: "luna.test".to_string(),
                        urls: vec![Url::from_str("https://luna.test?1").unwrap()],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        a.append(b);
        a.append(c);
        let term = Arc::new(AtomicBool::new(false));
        a.dedup(term);

        let c = Luna {
            programs: vec![
                Program {
                    name: "A".to_string(),
                    scopes: vec![Scope {
                        asset: ScopeType::Empty,
                        subs: vec![Sub {
                            asset: "luna.test".to_string(),
                            urls: vec![Url::from_str("https://luna.test?1").unwrap()],

                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Program {
                    name: "B".to_string(),
                    scopes: vec![],
                    ..Default::default()
                },
                Program {
                    name: "C".to_string(),
                    scopes: vec![],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(a.programs, c.programs);
    }
}
