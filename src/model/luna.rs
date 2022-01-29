use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::io::Write;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Luna {
    #[structopt(short, long)]
    pub name: String,
    #[structopt(short, long)]
    pub status: String,
    #[structopt(short, long)]
    pub version: String,
    #[structopt(skip)]
    pub counter: i32,
    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,
    #[structopt(short, long)]
    pub programs: Vec<Program>,
}

impl Luna {
    pub fn merge(&mut self, mut other: Self) {
        // Append
        self.counter += other.counter;
        self.programs.append(&mut other.programs);

        // Fill nones
        let luna_copy = self.clone();
        for i in 0..self.programs.len() {
            if self.programs[i].name.is_empty() {
                self.programs[i].set_name(&luna_copy);
            }
        }

        // Merge
        self.programs.sort();
        self.programs.dedup_by(Program::same_bucket);
    }
    /*
    pub fn get_rid_of_none(&mut self) {
        let mut commons = vec![];

        if let Some(scopes) = self
            .programs
            .iter()
            .find(|p| p.name == "none")
            .map(|p| &p.scopes)
        {
            for scope in scopes {
                if let Some(program) = self.program(&scope.asset) {
                    if !program.name.is_empty() {
                        commons.push((program.name.clone(), scope.clone()));
                    }
                }
            }
        }

        let mut luna = Self::new();

        for (p, s) in commons {
            let mut program = Program::new();
            program.name = p;
            program.scopes.push(s.clone());
            luna.programs.push(program);
            self.programs
                .iter_mut()
                .filter(|p| p.name == "none")
                .for_each(|p| p.scopes.retain(|sc| sc.asset != s.asset))
        }
        self.merge(luna);
    }

    pub fn remove_nones(&mut self) {
        let mut ip = 0;
        while ip < self.programs.len() {
            let mut is = 0;
            while is < self.programs[ip].scopes.len() {
                let mut isub = 0;
                while isub < self.programs[ip].scopes[is].subs.len() {
                    let mut ih = 0;
                    if self.programs[ip].scopes[is].subs[isub].asset.is_empty() {
                        while ih < self.programs[ip].scopes[is].subs[isub].hosts.len() {
                            if let Some(sub) =
                                self.sub(&self.programs[ip].scopes[is].subs[isub].hosts[ih].ip)
                            {
                                self.programs[ip].scopes[is].subs[isub].asset = sub.asset.clone();
                            }
                            ih += 1;
                        }
                    }

                    isub += 1;
                }
                is += 1;
            }
            ip += 1;
        }
    }
    */

    /*
    pub fn from_insert(insert: Insert) -> Self {
        let path = "luna.json";
        let mut luna = Self::new();

        match insert {
            Insert::Program(insert_program) => {
                luna.programs.push(insert_program.program);
            }
            Insert::Scope(insert_scope) => {
                let mut program = Program::new();
                program.name = insert_scope.program.unwrap_or_else(|| {
                    Luna::from_file(path)
                        .unwrap_or_default()
                        .get_program_name_from_scope(&insert_scope.scope.asset)
                        .unwrap_or_else(|| "none".to_string())
                });
                program.scopes.push(insert_scope.scope);
                program.scopes.extend(insert_scope.scopes);
                luna.programs.push(program);
            }
            Insert::Sub(insert_sub) => {
                let mut scope = Scope::new();
                scope.asset = insert_sub.scope.clone();
                scope.subs.push(insert_sub.sub);
                let mut program = Program::new();
                program.name = Luna::from_file(path)
                    .unwrap_or_default()
                    .get_program_name_from_scope(&insert_sub.scope)
                    .unwrap_or_else(|| insert_sub.program.unwrap_or_else(|| "none".to_string()));
                program.scopes.push(scope);
                luna.programs.push(program);
            }
            Insert::Url(insert_url) => {
                let mut sub = Sub::new();
                sub.asset = insert_url.sub.clone();
                sub.urls.push(insert_url.url);
                let scope_name = Luna::from_file(path)
                    .unwrap_or_default()
                    .get_scope_name_from_sub(&insert_url.sub)
                    .unwrap_or_else(|| "none".to_string());
                let mut scope = Scope::new();
                scope.asset = scope_name.clone();
                scope.subs.push(sub);

                let mut program = Program::new();
                program.name = Luna::from_file(path)
                    .unwrap_or_default()
                    .get_program_name_from_scope(&scope_name)
                    .unwrap_or_else(|| "none".to_string());
                program.scopes.push(scope);
                luna.programs.push(program);
            }
            Insert::Host(insert_host) => {
                let mut sub = Sub::new();
                sub.asset = insert_host.sub.clone();
                sub.hosts.push(insert_host.host);
                let scope_name = Luna::from_file(path)
                    .unwrap_or_default()
                    .get_scope_name_from_sub(&insert_host.sub)
                    .unwrap_or_else(|| "none".to_string());
                let mut scope = Scope::new();
                scope.asset = scope_name.clone();
                scope.subs.push(sub);

                let mut program = Program::new();
                program.name = Luna::from_file(path)
                    .unwrap_or_default()
                    .get_program_name_from_scope(&scope_name)
                    .unwrap_or_else(|| "none".to_string());
                program.scopes.push(scope);
                luna.programs.push(program);
            }
        }

        luna
    }
    */

    pub fn find(self, filter: &Filter) -> Vec<String> {
        /*
        let tech_filter = |t: &Tech| {
            (find.tech.is_none()
                || t.name
                    .to_lowercase()
                    .contains(&find.tech.as_ref().unwrap().to_lowercase()))
                && (find.tech_version.is_none() || find.tech_version == t.version)
        };

        let url_filter = |u: &Url| {
            (find.url.is_none()
                || u.url
                    .to_lowercase()
                    .contains(&find.url.as_ref().unwrap().to_lowercase()))
                && are_same(&find.title, &u.title)
                && (find.status_code.is_none() || find.status_code == u.status_code)
                && (find.content_type.is_none() || find.content_type == u.content_type)
                && (find.content_length.is_none() || find.content_length == u.content_length)
                && (find.tech.is_none() && find.tech_version.is_none()
                    || u.techs.iter().any(tech_filter))
        };

        let service_filter = |s: &Service| {
            (find.port.is_none()
                || s.port
                    .to_lowercase()
                    .contains(&find.port.as_ref().unwrap().to_lowercase()))
                && are_same(&find.service_name, &s.name)
        };
        let host_filter = |h: &Host| {
            (find.ip.is_none()
                || h.ip
                    .to_lowercase()
                    .contains(&find.ip.as_ref().unwrap().to_lowercase()))
                && (find.port.is_none() && find.service_name.is_none()
                    || h.services.iter().any(service_filter))
        };
        let sub_filter = |s: &Sub| {
            (find.sub.is_none()
                || s.asset
                    .to_lowercase()
                    .contains(&find.sub.as_ref().unwrap().to_lowercase()))
                && (find.ip.is_none() && find.port.is_none() && find.service_name.is_none()
                    || s.hosts.iter().any(host_filter))
                && (find.url.is_none()
                    && find.title.is_none()
                    && find.status_code.is_none()
                    && find.content_type.is_none()
                    && find.content_length.is_none()
                    && find.tech.is_none()
                    && find.tech_version.is_none()
                    || s.urls.iter().any(url_filter))
        };

        let scope_filter = |s: &Scope| {
            (find.scope.is_none()
                || s.asset
                    .to_lowercase()
                    .contains(&find.scope.as_ref().unwrap().to_lowercase()))
                && are_same(&find.scope_type, &s.typ)
                && (find.scope_bounty.is_none() || find.scope_bounty == s.bounty)
                && (find.sub.is_none()
                    && find.ip.is_none()
                    && find.port.is_none()
                    && find.service_name.is_none()
                    && find.url.is_none()
                    && find.title.is_none()
                    && find.status_code.is_none()
                    && find.content_type.is_none()
                    && find.content_length.is_none()
                    && find.tech.is_none()
                    && find.tech_version.is_none()
                    || s.subs.iter().any(sub_filter))
        };

        let program_filter = |p: &Program| {
            (find.program.is_none()
                || p.name
                    .to_lowercase()
                    .contains(&find.program.as_ref().unwrap().to_lowercase()))
                && are_same(&find.program_platform, &p.platform)
                && are_same(&find.program_type, &p.typ)
                && (find.program_bounty.is_none() || find.program_bounty == p.bounty)
                && (find.program_state.is_none() || find.program_state == p.state)
                && ((find.scope.is_none()
                    && find.scope_type.is_none()
                    && find.scope_bounty.is_none()
                    && find.sub.is_none()
                    && find.ip.is_none()
                    && find.port.is_none()
                    && find.service_name.is_none()
                    && find.url.is_none()
                    && find.title.is_none()
                    && find.status_code.is_none()
                    && find.content_type.is_none()
                    && find.content_length.is_none()
                    && find.tech.is_none()
                    && find.tech_version.is_none())
                    || p.scopes.iter().any(scope_filter))
        };
        */

        match filter.field {
            Fields::Program => self
                .programs
                .into_iter()
                .filter(|p| p.matches(filter))
                .map(|p| p.name)
                .collect(),
            Fields::Scope => self
                .programs
                .into_iter()
                .filter(|p| p.matches(filter))
                .map(|p| p.scopes)
                .flatten()
                .into_iter()
                .filter(|s| s.matches(filter))
                .map(|s| s.asset)
                .collect(),

            Fields::Sub => self
                .programs
                .into_iter()
                .filter(|p| p.matches(filter))
                .map(|p| p.scopes)
                .flatten()
                .into_iter()
                .filter(|s| s.matches(filter))
                .map(|s| s.subs)
                .flatten()
                .into_iter()
                .filter(|s| s.matches(filter))
                .map(|s| s.asset)
                .collect(),
            Fields::URL => self
                .programs
                .into_iter()
                .filter(|p| p.matches(filter))
                .map(|p| p.scopes)
                .flatten()
                .into_iter()
                .filter(|s| s.matches(filter))
                .map(|s| s.subs)
                .flatten()
                .into_iter()
                .filter(|s| s.matches(filter))
                .map(|s| s.urls)
                .flatten()
                .filter(|u| u.matches(filter))
                .map(|u| u.url)
                .collect(),
        }

        // luna.programs.iter().find(|x|x.eq(slef.))
    }

    pub fn program(&self, scope: &str) -> Option<&Program> {
        self.programs
            .iter()
            .filter(|p| !p.name.is_empty())
            .find(|p| p.scopes.iter().any(|s| s.asset == scope))
    }

    pub fn scope(&self, sub: &str) -> Option<&Scope> {
        self.programs
            .iter()
            .map(|p| &p.scopes)
            .flatten()
            .filter(|s| !s.asset.is_empty())
            .find(|s| s.subs.iter().any(|s| s.asset == sub))
    }
    pub fn sub(&self, ip: &str) -> Option<&Sub> {
        self.programs
            .iter()
            .map(|p| &p.scopes)
            .flatten()
            .map(|s| &s.subs)
            .flatten()
            .filter(|s| !s.asset.is_empty())
            .find(|s| s.hosts.iter().any(|h| h.ip == ip))
    }

    pub fn save(&self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let str = serde_json::to_string(&self).unwrap();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        Ok(file.write(str.as_bytes())?)
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file)?)
    }

    pub fn new() -> Self {
        Default::default()
    }
}

impl From<InsertProgram> for Luna {
    fn from(i: InsertProgram) -> Self {
        Luna {
            programs: vec![Program {
                update: Some(Utc::now()),
                ..i.program
            }],
            update: Some(Utc::now()),
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
                    ..i.scope
                }],
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
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
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
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
                        ..i.sub
                    }],
                    update: Some(Utc::now()),
                    ..Default::default()
                }],
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
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
                    update: Some(Utc::now()),
                    ..Default::default()
                }],
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
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
                            ..i.url
                        }],
                        update: Some(Utc::now()),
                        ..Default::default()
                    }],
                    update: Some(Utc::now()),
                    ..Default::default()
                }],
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
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
                        update: Some(Utc::now()),
                        ..Default::default()
                    }],
                    update: Some(Utc::now()),
                    ..Default::default()
                }],
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
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
                            ..i.host
                        }],
                        update: Some(Utc::now()),
                        ..Default::default()
                    }],
                    update: Some(Utc::now()),
                    ..Default::default()
                }],
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
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
                        update: Some(Utc::now()),
                        ..Default::default()
                    }],
                    update: Some(Utc::now()),
                    ..Default::default()
                }],
                update: Some(Utc::now()),
                ..Default::default()
            }],
            update: Some(Utc::now()),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn rw_file() {
        assert_eq!(Luna::new().save("test.json").unwrap(), 74);
        assert_eq!(Luna::from_file("test.json").unwrap(), Luna::new());
    }
}
