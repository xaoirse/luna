use super::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
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
        self.programs
            .par_iter_mut()
            .for_each(|p| p.set_name(&luna_copy));

        // Merge
        self.programs.par_sort();
        self.programs.dedup_by(Program::same_bucket);
    }

    pub fn find(&self, filter: &Filter) -> Vec<String> {
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
                .map(|s| match filter.verbose {
                    0 => s.asset.to_string(),
                    _ => format!("{:#?}", s),
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
                .map(|s| match filter.verbose {
                    0 => s.asset.to_string(),
                    _ => format!("{:#?}", s),
                })
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
                .map(|u| match filter.verbose {
                    0 => u.url.to_string(),
                    _ => format!("{:#?}", u),
                })
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
                .map(|h| match filter.verbose {
                    0 => h.ip.to_string(),
                    _ => format!("{:#?}", h),
                })
                .collect(),
            Fields::None => vec!["".to_string()],
            Fields::Service => todo!(),
            Fields::Tech => todo!(),
            Fields::Keyword => todo!(),
        }
    }
    pub fn find_all(&self, field: Fields) -> Vec<String> {
        self.find(&Filter {
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

    pub fn save(&self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        std::fs::copy(path, &format!("{}{}", path, Utc::now().to_rfc2822()))?;
        let str = serde_json::to_string(&self)?;
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn rw_file() {
        assert_eq!(Luna::default().save("test.json").unwrap(), 74);
        assert_eq!(Luna::from_file("test.json").unwrap(), Luna::default());
    }
}
