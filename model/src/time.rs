use super::Errors;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Time(pub DateTime<Utc>);
impl FromStr for Time {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match DateTime::parse_from_rfc3339(s) {
            Ok(date) => Ok(Self(date.with_timezone(&Utc::now().timezone()))),
            Err(_) => Err("Parse error".into()),
        }
    }
}
impl Default for Time {
    fn default() -> Self {
        Self(Utc::now())
    }
}
