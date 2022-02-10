use crate::model::{Errors, Fields, Filter, Luna};
use log::{debug, error};
use rayon::prelude::*;
use rayon::{iter::Map, vec::IntoIter};
use regex::Regex;
use std::{error, fmt, process::Command};

#[derive(Debug)]
pub enum Error {
    Pattern(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pattern(msg) => write!(f, "{}", msg),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Self::Pattern(msg) => msg,
        }
    }
}

pub struct Data {
    pub input: String,
    pub field: Fields,
    pub output: String,
}
impl Data {
    fn parse(&self, regex: &Regex) -> Vec<Luna> {
        regex
            .captures_iter(&self.output)
            .map(|caps| {
                let get = |key| caps.name(key).map(|v| v.as_str().to_string());

                let mut luna = Filter {
                    field: self.field,
                    verbose: 0,

                    program: get("program"),
                    program_platform: get("program_platform"),
                    program_handle: get("program_handle"),
                    program_type: get("program_type"),
                    program_url: get("program_url"),
                    program_icon: get("program_icon"),
                    program_bounty: get("program_bounty"),
                    program_state: get("program_state"),

                    scope: get("scope"),
                    scope_type: get("scope_type"),
                    scope_bounty: get("scope_bounty"),
                    scope_severity: get("scop_severity"),

                    sub: get("sub"),
                    sub_typ: get("sub_type"),

                    ip: get("ip"),

                    port: get("port"),
                    service_name: get("service_name"),
                    service_protocol: get("service_protocol"),
                    service_banner: get("service_banner"),

                    url: get("url"),
                    title: get("title"),
                    status_code: get("status_code"),
                    response: get("response"),

                    tech: get("tech"),
                    tech_version: get("tech_version"),

                    updated_at: None,
                    started_at: None,
                };
                let input = Some(self.input.clone());
                match self.field {
                    Fields::Program => luna.program = input,
                    Fields::Scope => luna.scope = input,
                    Fields::Sub => luna.sub = input,
                    Fields::Url => luna.url = input,
                    Fields::IP => luna.ip = input,
                    Fields::Service => luna.port = input,
                    Fields::Tech => luna.tech = input,
                    Fields::Keyword => todo!(),
                    Fields::None => todo!(),
                }
                luna.into()
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Script {
    pub regex: Regex,
    pub command: String,
    pub field: Fields,
}

impl Script {
    fn execute<'a>(
        &'a self,
        luna: &Luna,
    ) -> Map<IntoIter<String>, impl Fn(String) -> Result<Data, Errors> + 'a> {
        luna.find_all(self.field).into_par_iter().map(|input| {
            let cmd = self.command.replace(&self.field.substitution(), &input);
            debug!("Command: {}", cmd);
            let output = String::from_utf8(Command::new("sh").arg("-c").arg(cmd).output()?.stdout)?;
            Ok(Data {
                input,
                field: self.field,
                output,
            })
        })
    }
}

#[derive(Debug, Default)]
pub struct Scripts {
    pub scripts: Vec<Script>,
    // TODO other options
}

impl Scripts {
    pub fn run(self, luna: &mut Luna) {
        self.scripts
            .into_iter() // No parallel here for preserving order of scripts
            .for_each(|script| {
                script
                    .execute(luna)
                    .filter_map(|result| match result {
                        Ok(data) => Some(data),
                        Err(err) => {
                            error!("Error in executing: {}", err);
                            None
                        }
                    })
                    .flat_map(|data| data.parse(&script.regex))
                    .collect::<Vec<Luna>>() // This should run parallel here
                    .into_iter()
                    .for_each(|l| luna.append(l))
            });
    }
}

#[allow(clippy::blocks_in_if_conditions)]
pub fn parse(path: String) -> Result<Scripts, Errors> {
    let mut scripts = vec![];
    let mut pattern = String::new();

    for line in std::fs::read_to_string(path)?.trim().lines() {
        if line.trim().starts_with("pattern") {
            pattern = line
                .split_once("=")
                .map_or("".to_string(), |p| p.1.trim().to_string())
        } else if line.trim().chars().next().map_or(false, |c| {
            c.is_ascii_alphabetic() || '.' == c || '/' == c || '\\' == c
        }) {
            if pattern.is_empty() {
                return Err(Box::new(Error::Pattern(
                    "Where the fuck is first pattern?".to_string(),
                )));
            }

            let field = if line.contains("${program}") {
                Fields::Program
            } else if line.contains("${scope}") {
                Fields::Scope
            } else if line.contains("${sub}") {
                Fields::Sub
            } else if line.contains("${url}") {
                Fields::Url
            } else if line.contains("${ip}") {
                Fields::IP
            } else if line.contains("${port}") {
                Fields::Service
            } else if line.contains("${tech}") {
                Fields::Tech
            } else if line.contains("${keyword}") {
                Fields::Keyword
            } else {
                Fields::None
            };

            if let Ok(regex) = regex::Regex::new(&pattern) {
                if !regex_check(&regex) {
                    return Err(Box::new(Error::Pattern(
                    "Pattern doesn't have any of necessery names \"program, scope, sub, url, ip\""
                        .to_string(),
                )));
                }

                scripts.push(Script {
                    regex,
                    command: line.trim().to_string(),
                    field,
                })
            } else {
                error!("Fucking pattern: {}", pattern);
                panic!("Fucking pattern: {}", pattern);
            }
        }
    }

    Ok(Scripts { scripts })
}

fn regex_check(regex: &Regex) -> bool {
    regex
        .capture_names()
        .flatten()
        .any(|name| name == "program" || name == "scope" || name == "sub" || name == "host")
}
