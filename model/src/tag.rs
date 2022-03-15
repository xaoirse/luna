use std::str::FromStr;

use super::*;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Parser, Default, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    #[clap(long)]
    pub severity: Option<String>,
    #[clap(long)]
    pub values: Vec<String>,

    #[clap(skip)]
    pub update: Time,
    #[clap(skip)]
    pub start: Time,
}

impl FromStr for Tag {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: s.to_string(),
            severity: None,
            values: vec![],
            update: Time::default(),
            start: Time::default(),
        })
    }
}
impl Tag {
    pub fn merge(&mut self, other: Self) {
        let new = self.update < other.update;

        merge(&mut self.severity, other.severity, new);

        self.update = self.update.max(other.update);
        self.start = self.start.min(other.start);

        for value in other.values {
            if !self.values.iter_mut().any(|t| t == &value) {
                self.values.push(value);
            }
        }
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!(
                "{} [{}]",
                self.name,
                self.severity.as_ref().map_or("", |s| s),
            ),
            2 => format!(
                "{} [{}] [{}]",
                self.name,
                self.severity.as_ref().map_or("", |s| s),
                self.values.join(", ")
            ),
            3 => format!(
                "{} [{}]
    Values: [{}{}
    Update: {}
    Start:  {}
    ",
                self.name,
                self.severity.as_ref().map_or("", |s| s),
                self.values
                    .iter()
                    .map(|s| format!("\n        {}", s))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.values.is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
                self.update.0.to_rfc2822(),
                self.start.0.to_rfc2822(),
            ),

            _ => format!("{:#?}", self),
        }
    }
}
