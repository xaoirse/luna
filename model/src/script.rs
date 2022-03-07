use clap::Parser;
use fixed_buffer::{deframe_line, FixedBuf};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use log::{debug, error, warn};
use rayon::prelude::*;
use regex::bytes::Regex;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::*;

#[derive(Debug)]
pub struct Data<'a> {
    pub input: &'a str,
    pub field: Fields,
    pub output: &'a [u8],
}
impl<'a> Data<'a> {
    fn parse(&self, regex: &Regex) -> Luna {
        regex
            .captures_iter(self.output)
            .filter_map(|caps| {
                let get = |key| {
                    caps.name(key)
                        .map(|v| String::from_utf8_lossy(v.as_bytes()).to_string())
                };

                let mut luna = Filter {
                    n: None,

                    program: get("program"),
                    program_platform: get("program_platform"),
                    program_handle: get("program_handle"),
                    program_type: get("program_type"),
                    program_url: get("program_url"),
                    program_icon: get("program_icon"),
                    program_bounty: get("program_bounty"),
                    program_state: get("program_state"),

                    scope: get("scope"),
                    scope_bounty: get("scope_bounty"),
                    scope_severity: get("scop_severity"),

                    sub: get("sub"),
                    sub_type: get("sub_type"),

                    ip: get("ip"),

                    port: get("port"),
                    service_name: get("service_name"),
                    service_protocol: get("service_protocol"),
                    service_banner: get("service_banner"),

                    url: get("url"),
                    title: get("title"),
                    status_code: get("status_code"),
                    response: get("response"),

                    tag: get("tag"),
                    tag_severity: get("tag_severity"),
                    tag_value: get("tag_value"),

                    updated_at: None,
                    started_at: None,
                };
                let input = Some(self.input.to_string());
                match self.field {
                    Fields::Luna => (),
                    Fields::Program => luna.program = input,
                    Fields::Domain => luna.scope = input,
                    Fields::Cidr => luna.scope = input,
                    Fields::Sub => luna.sub = input,
                    Fields::Url => luna.url = input,
                    Fields::IP => luna.ip = input,
                    Fields::Service => luna.port = input,
                    Fields::Tag => luna.tag = input,
                    Fields::Keyword => (),
                    Fields::None => (),
                }
                debug!("{:#?}", luna);

                // Filter orphan fields
                if (luna.program.is_some()
                    || luna.scope.is_some()
                    || luna.sub.is_some()
                    || luna.ip.is_some()
                    || luna.url.is_some()
                    || luna.program.is_some()
                        == (luna.program_platform.is_some()
                            || luna.program_handle.is_some()
                            || luna.program_type.is_some()
                            || luna.program_url.is_some()
                            || luna.program_icon.is_some()
                            || luna.program_bounty.is_some()
                            || luna.program_state.is_some()))
                    && (luna.scope.is_some()
                        || luna.ip.is_some()
                        || luna.url.is_some()
                        || luna.scope.is_some()
                        || luna.sub.is_none()
                            == (luna.scope_bounty.is_some() || luna.scope_severity.is_some()))
                    && (luna.sub.is_some()
                        || luna.ip.is_some()
                        || luna.url.is_some()
                        || luna.sub.is_some() == luna.sub_type.is_some())
                    && (luna.url.is_some()
                        || luna.url.is_some()
                            == (luna.title.is_some()
                                || luna.status_code.is_some()
                                || luna.response.is_some()))
                {
                    Some(luna.into())
                } else {
                    warn!("Fucking Orphan detected: {:#?}", luna);
                    None
                }
            })
            .fold(Luna::default(), |mut init, l| {
                init.append(l);
                init
            })
    }
}

#[derive(Debug)]
pub struct Script {
    pub verbose: u8,
    pub cd: String,
    pub regex: Regex,
    pub command: String,
    pub field: Fields,
}

impl Script {
    fn execute(&self, luna: &Luna, filter: &FilterRegex, term: Arc<AtomicBool>) -> Luna {
        let elements = luna.find(self.field, filter, 0);

        let ps = ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} {wide_msg}\n[{elapsed_precise}] [{wide_bar:.cyan/cyan}] {pos}/{len} ({eta})",
                    ).unwrap()
                    .progress_chars("▓█░");

        let pb = ProgressBar::new(elements.len().min(filter.n) as u64);

        pb.set_style(ps);

        if self.verbose == 0 {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        } else {
            pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
        }

        if self.verbose > 1 {
            pb.clone()
                .with_finish(ProgressFinish::WithMessage(self.command.clone().into()));
        }

        pb.set_message(self.command.clone());

        elements
            .par_iter()
            .take(filter.n)
            .filter_map(|input| {
                if term.load(Ordering::Relaxed) {
                    warn!("Command aborted! {} => {}", input, self.command);
                    return None;
                }

                let cmd = self.command.replace(&self.field.substitution(), input);
                debug!("Command: {}", &cmd);

                let child = match Command::new("sh")
                    .current_dir(&self.cd)
                    .arg("-c")
                    .arg(&cmd)
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(err) => {
                        error!("{err}");
                        return None;
                    }
                };

                if let Some(mut stdout) = child.stdout {
                    const S: usize = 4096;
                    let mut buf: FixedBuf<S> = FixedBuf::new();

                    let mut luna = Luna::default();

                    while let Ok(Some(bytes)) = buf.read_frame(&mut stdout, deframe_line) {
                        luna.append(
                            Data {
                                input,
                                field: self.field,
                                output: bytes,
                            }
                            .parse(&self.regex),
                        );
                    }

                    pb.set_message(cmd);
                    pb.inc(1);

                    debug!("{:#?}", luna);

                    Some(luna)
                } else {
                    error!("Executing: {cmd}");
                    None
                }
            })
            .reduce(Luna::default, |mut init, l| {
                init.append(l);

                if term.load(Ordering::Relaxed) {
                    return init;
                }

                init
            })
    }
}

#[derive(Debug)]
pub struct Scripts {
    pub scripts: Vec<Script>,

    pub filter: FilterRegex,
}

impl Scripts {
    pub fn run(self, luna: &mut Luna, term: Arc<AtomicBool>) {
        self.scripts
            .into_iter() // No parallel here for preserving order of scripts
            .for_each(|script| {
                if term.load(Ordering::Relaxed) {
                    return;
                }
                let l = script.execute(luna, &self.filter, term.clone());

                luna.append(l);

                if term.load(Ordering::Relaxed) {
                    luna.save_as("backup.json").unwrap();
                    return;
                }

                luna.dedup(term.clone());
                luna.save();
            })
    }
}

#[derive(Debug, Parser)]
pub struct ScriptCli {
    pub path: String,

    #[clap(short, long, parse(from_occurrences), help = "Show progress bar")]
    pub verbose: u8,

    #[clap(flatten)]
    pub filter: Filter,
}

impl ScriptCli {
    #[allow(clippy::blocks_in_if_conditions)]
    pub fn parse(self) -> Result<Scripts, Errors> {
        let mut scripts = vec![];
        let mut regex = String::new();
        let regex_pat = Regex::new(r"(?:^#\s)*regex\s*=")?;

        for (n, line) in std::fs::read_to_string(&self.path)?
            .trim()
            .lines()
            .enumerate()
        {
            if regex_pat.is_match(line.as_bytes()) {
                regex = line
                    .split_once("=")
                    .map_or("".to_string(), |p| p.1.trim().to_string())
            } else if line.trim().chars().next().map_or(false, |c| {
                c.is_ascii_alphabetic() || '.' == c || '/' == c || '\\' == c
            }) {
                if regex.is_empty() {
                    return Err("Where the fuck is the first regex?".into());
                }

                let field = if line.contains("${program}") {
                    Fields::Program
                } else if line.contains("${domain}") {
                    Fields::Domain
                } else if line.contains("${cidr}") {
                    Fields::Cidr
                } else if line.contains("${sub}") {
                    Fields::Sub
                } else if line.contains("${url}") {
                    Fields::Url
                } else if line.contains("${ip}") {
                    Fields::IP
                } else if line.contains("${port}") {
                    Fields::Service
                } else if line.contains("${tag}") {
                    Fields::Tag
                } else if line.contains("${keyword}") {
                    Fields::Keyword
                } else {
                    // TODO check if ${invalid}
                    Fields::None
                };

                if let Ok(regex) = Regex::new(&regex) {
                    if !regex_check(&regex) {
                        return Err(
                        format!("Line {} regex \"{}\"  doesn't have necessery names \"program\", \"scope\", \"sub\", \"url\" or \"ip\""
                            ,n,regex).into(),
                    );
                    }

                    let mut cd = std::path::Path::new(&self.path)
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    if cd.is_empty() {
                        cd = ".".to_string()
                    }

                    let script = Script {
                        verbose: self.verbose,
                        cd,
                        regex,
                        command: line.trim().to_string(),
                        field,
                    };
                    debug!("{:#?}", script);
                    scripts.push(script)
                } else {
                    error!("Fucking regex: {}", regex);
                    panic!("Fucking regex: {}", regex);
                }
            }
        }

        let filter: FilterRegex = self.filter.try_into()?;
        Ok(Scripts { scripts, filter })
    }
}
fn regex_check(regex: &Regex) -> bool {
    let names: Vec<_> = regex.capture_names().flatten().collect();

    (names.contains(&"program")
        || names.contains(&"scope")
        || names.contains(&"sub")
        || names.contains(&"ip")
        || names.contains(&"url")
        || names.contains(&"program")
            == (names.contains(&"program_platform")
                || names.contains(&"program_handle")
                || names.contains(&"program_type")
                || names.contains(&"program_url")
                || names.contains(&"program_icon")
                || names.contains(&"program_bounty")
                || names.contains(&"program_state")))
        && (names.contains(&"scope")
            || names.contains(&"sub")
            || names.contains(&"ip")
            || names.contains(&"url")
            || names.contains(&"scope")
                == (names.contains(&"scope_bounty") || names.contains(&"scope_severity")))
        && (names.contains(&"sub")
            || names.contains(&"ip")
            || names.contains(&"url")
            || names.contains(&"sub") == names.contains(&"sub_type"))
        && (names.contains(&"port")
            || names.contains(&"port")
                == (names.contains(&"service_name")
                    || names.contains(&"service_protocol")
                    || names.contains(&"service_banner")))
        && (names.contains(&"url")
            || names.contains(&"url")
                == (names.contains(&"title")
                    || names.contains(&"status_code")
                    || names.contains(&"response")))
        && (names.contains(&"tag")
            || names.contains(&"tag")
                == (names.contains(&"tag_severity") || names.contains(&"tag_value")))
}
