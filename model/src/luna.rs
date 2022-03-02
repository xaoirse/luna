use super::*;
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
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Luna {
    pub fn append(&mut self, mut other: Self) {
        self.counter += other.counter;
        self.programs.append(&mut other.programs);

        self.update = self.update.max(other.update);
        self.start = self.start.min(other.start);
    }

    pub fn dedup(&mut self, _: Arc<AtomicBool>) {
        //////////////
        ///// Hosts
        /////////////////////
        let mut hosts: Vec<&mut Host> = self
            .programs
            .iter_mut()
            .flat_map(|p| &mut p.scopes)
            .flat_map(|s| &mut s.subs)
            .flat_map(|s| &mut s.hosts)
            .collect();

        hosts.par_sort();
        for i in (1..hosts.len()).rev() {
            if hosts[i] == hosts[i - 1] {
                let (a, b) = hosts.split_at_mut(i);
                Host::same_bucket(b[0], a[i - 1]);
                b[0].ip.clear();
            }
        }
        hosts.iter_mut().for_each(|h| {
            h.services.par_sort();
            for i in (1..h.services.len()).rev() {
                if h.services[i] == h.services[i - 1] {
                    let (a, b) = h.services.split_at_mut(i);
                    Service::same_bucket(&mut b[0], &mut a[i - 1]);
                    b[0].port.clear();
                }
            }

            h.services.retain(|srv| !srv.is_empty());
        });

        //////////////
        ///// Urls
        /////////////////////
        let mut urls: Vec<&mut Url> = self
            .programs
            .iter_mut()
            .flat_map(|p| &mut p.scopes)
            .flat_map(|s| &mut s.subs)
            .flat_map(|s| &mut s.urls)
            .collect();

        urls.par_sort();
        for i in (1..urls.len()).rev() {
            if urls[i] == urls[i - 1] {
                let (a, b) = urls.split_at_mut(i);
                Url::same_bucket(b[0], a[i - 1]);
                b[0].url.clear();
            }
        }
        urls.iter_mut().for_each(|u| {
            u.tags.par_sort();
            for i in (1..u.tags.len()).rev() {
                if u.tags[i] == u.tags[i - 1] {
                    let (a, b) = u.tags.split_at_mut(i);
                    Tag::same_bucket(&mut b[0], &mut a[i - 1]);
                    b[0].name.clear();
                }
            }
            u.tags
                .iter_mut()
                .for_each(|t| t.values.retain(|vlu| !vlu.is_empty()));

            u.tags.retain(|tag| !tag.is_empty());
        });

        //////////////
        ///// Subs
        /////////////////////
        let mut subs: Vec<&mut Sub> = self
            .programs
            .iter_mut()
            .flat_map(|p| &mut p.scopes)
            .flat_map(|s| {
                if let ScopeType::Domain(d) = &s.asset {
                    s.subs.retain(|sub| sub.asset.contains(d));
                }
                &mut s.subs
            })
            .collect();

        subs.par_sort();
        for i in (1..subs.len()).rev() {
            if subs[i] == subs[i - 1] {
                let (a, b) = subs.split_at_mut(i);
                Sub::same_bucket(b[0], a[i - 1]);
                b[0].asset.clear();
            }
        }
        subs.iter_mut()
            .for_each(|s| s.urls.retain(|url| !url.is_empty()));
        subs.iter_mut()
            .for_each(|s| s.hosts.retain(|host| !host.is_empty()));

        //////////////
        ///// Scopes
        /////////////////////
        let mut scopes: Vec<&mut Scope> = self
            .programs
            .iter_mut()
            .flat_map(|p| &mut p.scopes)
            .collect();

        scopes.par_sort();
        for i in (1..scopes.len()).rev() {
            if scopes[i] == scopes[i - 1] {
                let (a, b) = scopes.split_at_mut(i);
                Scope::same_bucket(b[0], a[i - 1]);
                b[0].asset = ScopeType::Empty;
            }
        }
        scopes
            .iter_mut()
            .for_each(|s| s.subs.retain(|sub| !sub.is_empty()));

        //////////////
        ///// Programs
        /////////////////////
        self.programs.par_sort();
        for i in (1..self.programs.len()).rev() {
            if self.programs[i] == self.programs[i - 1] {
                let (a, b) = self.programs.split_at_mut(i);
                Program::same_bucket(&mut b[0], &mut a[i - 1]);
                b[0].name.clear();
            }
        }
        self.programs
            .iter_mut()
            .for_each(|s| s.scopes.retain(|scp| !scp.is_empty()));

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
                                d.parse::<cidr::IpCidr>()
                                    .unwrap()
                                    .iter()
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

    fn save_as(&self, path: &str) -> Result<usize, Errors> {
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
            Insert::Tag(i) => i.into(),
            Insert::Service(i) => i.into(),
        }
    }
}

impl From<Filter> for Luna {
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
        } else {
            vec![Url {
                url: f.url.take().unwrap_or_default(),
                title: f.title.take(),
                status_code: f.status_code.take(),
                response: f.response.take(),
                tags,
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
            }]
        };

        let scopes = if scope_is_none {
            vec![]
        } else if let (Some(scope), Some(sub)) = (f.scope.as_ref(), subs.first()) {
            if sub.asset.contains(scope) {
                vec![Scope {
                    asset: ScopeType::Domain(scope.to_string()),
                    severity: f.scope_severity,
                    bounty: f.scope_bounty,
                    subs,
                    update: Some(Utc::now()),
                    start: Some(Utc::now()),
                }]
            } else {
                vec![Scope {
                    asset: ScopeType::Empty,
                    severity: f.scope_severity,
                    bounty: f.scope_bounty,
                    subs,
                    update: Some(Utc::now()),
                    start: Some(Utc::now()),
                }]
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

        const N: usize = 1000;
        const M: usize = 901; // 10 contains 1

        let mut luna = Luna::default();

        for i in 0..N {
            let l = Luna {
                programs: vec![Program {
                    name: "S".to_string(),
                    scopes: vec![Scope {
                        asset: ScopeType::Empty,
                        subs: vec![Sub {
                            asset: "luna.test".to_string(),
                            urls: vec![Url {
                                url: format!("https://luna.test?{}", i),
                                ..Default::default()
                            }],
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
                        urls: vec![Url {
                            url: "https://luna.test?1".to_string(),
                            ..Default::default()
                        }],
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
                        urls: vec![Url {
                            url: "https://luna.test?1".to_string(),
                            ..Default::default()
                        }],
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
                        urls: vec![Url {
                            url: "https://luna.test?3".to_string(),
                            ..Default::default()
                        }],
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
                            urls: vec![
                                Url {
                                    url: "https://luna.test?1".to_string(),
                                    ..Default::default()
                                },
                                Url {
                                    url: "https://luna.test?3".to_string(),
                                    ..Default::default()
                                },
                            ],
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
