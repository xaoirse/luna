use log::{debug, error};
use rayon::prelude::*;
use regex::Regex;
use std::{
    error, fmt,
    process::{Command, Output},
};

use super::run::{Fields, InsertProgram};
use crate::model::{Luna, Program};

macro_rules! regex {
    ($re:ident $(,)?) => {{
        static RE: once_cell::sync::OnceCell<Result<regex::Regex, regex::Error>> =
            once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re.as_str()))
    }};
}

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

type LunaResult = Result<Luna, Box<dyn std::error::Error + Send + Sync>>;

pub struct Data {
    pub input: String,
    pub field: Fields,
    pub output: Result<Output, std::io::Error>,
}
impl Data {
    fn into_luna(self, regex: &Regex) -> LunaResult {
        if let Some(caps) = regex.captures(&String::from_utf8(self.output?.stdout)?) {
            let get = |key| caps.name(key).map(|v| v.as_str().to_string());

            let luna: Luna = match self.field {
                Fields::Program => InsertProgram {
                    program: Program {
                        name: self.input,
                        url: get("program_url"),
                        ..Default::default()
                    },
                }
                .into(),
                Fields::Scope => todo!(),
                Fields::Sub => todo!(),
                Fields::Url => todo!(),
                Fields::IP => todo!(),
                Fields::Keyword => todo!(),
                Fields::Service => todo!(),
                Fields::None => todo!(),
                Fields::Tech => todo!(),
            };
            Ok(luna)
        } else {
            Err(Box::new(Error::Pattern("No match".to_string())))
        }
    }
}

#[derive(Debug)]
pub struct Script {
    pub regex: &'static Regex,
    pub command: String,
    pub field: Fields,
}

impl Script {
    fn execute(&self, luna: &Luna) -> Vec<Data> {
        luna.find_all(self.field)
            .into_par_iter()
            .map(|input| {
                let cmd = self.command.replace(&self.field.substitution(), &input);
                debug!("Command: {}", cmd);
                let output = Command::new("sh").arg("-c").arg(cmd).output();
                Data {
                    input,
                    field: self.field,
                    output,
                }
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Scripts {
    pub scripts: Vec<Script>,
    // TODO other options
}

impl Scripts {
    pub fn run(self, luna: &Luna) -> Vec<LunaResult> {
        self.scripts
            .into_iter() // No parallel here for preserving order of scripts
            .flat_map(|script| {
                script
                    .execute(luna)
                    .into_par_iter()
                    .map(|data| data.into_luna(script.regex))
                    .collect::<Vec<LunaResult>>()
            })
            .collect()
    }
}

pub fn parse(path: String) -> Result<Scripts, Box<dyn error::Error>> {
    let mut scripts = vec![];
    let mut pattern = String::new();

    for line in std::fs::read_to_string(path)?.trim().lines() {
        if line.trim().starts_with("pattern") {
            pattern = line
                .split_once("=")
                .map_or("".to_string(), |p| p.1.trim().to_string())
        } else if line
            .trim()
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_alphabetic())
        {
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

            if let Ok(regex) = regex!(pattern).as_ref() {
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

// sub
// fn _regex() -> Regex {
//     static PAT: &str =
//         r"(?:^|//|\s)((?:[0-9\-a-z]+\.)+[0-9a-z][0-9\-a-z]*[0-9a-z])(?:$|[\D\W])";
//     lazy_static! {
//         static ref RE: Regex = regex::RegexBuilder::new(PAT)
//             .multi_line(true)
//             .build()
//             .unwrap();
//     }
//     RE.clone()
// }

//url
// fn _regex() -> Regex {
//     // TODO scheme is in match 1
//     static PAT: &str = r"(\w+)://[-a-zA-Z0-9:@;?&=/%\+\.\*!'\(\),\$_\{\}\^~\[\]`#|]+";
//     lazy_static! {
//         static ref RE: Regex = regex::RegexBuilder::new(PAT)
//             .multi_line(true)
//             .build()
//             .unwrap();
//     }
//     RE.clone()
// }

// host
// fn _regex() -> Regex {
//     static PAT: &str = r"(?:^|//|\s|\b)((?:[0-9\-a-z]+\.)+[0-9a-z][0-9\-a-z]*[0-9a-z])[\D\W]*((?:[0-9]{1,3}\.){3}[0-9]{1,3})(?:$|[\D\W\s])";
//     lazy_static! {
//         static ref RE: Regex = regex::RegexBuilder::new(PAT)
//             .multi_line(true)
//             .build()
//             .unwrap();
//     }
//     RE.clone()
// }

// async fn notif(self) -> bool {
//     let text = self.to_string();
//     let text = text.trim();
//     if !text.is_empty() {
//         if let Some(url) = crate::env::get("DISCORD") {
//             let cli = reqwest::Client::builder().build().unwrap();
//             match cli
//                 .post(url)
//                 .header("Content-Type", "application/json")
//                 .body(format!(r#"{{"username": "Luna", "content": "{}"}}"#, text))
//                 .send()
//                 .await
//             {
//                 Ok(resp) => {
//                     if resp.status() == 204 {
//                         return true;
//                     }
//                 }
//                 Err(err) => err.error(),
//             };
//         }
//     }
//     false
// }
