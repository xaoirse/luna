use chrono::{DateTime, Local, Utc};
use cidr::IpCidr;
use clap::{ArgEnum, Parser};
use fixed_buffer::{deframe_line, FixedBuf};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{
    fmt::{self, Display},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    str::FromStr,
};
use url::Host;

pub mod asset;
pub mod filter;
pub mod luna;
pub mod program;
pub mod request;
pub mod script;
pub mod serde_cidr;
pub mod tag;
pub mod time;

pub use asset::*;
pub use filter::*;
pub use luna::Luna;
pub use program::Program;
pub use request::Request;
pub use script::ScriptCli;
pub use tag::Tag;
pub use time::Time;

pub type Errors = Box<dyn std::error::Error + Sync + Send>;

pub fn date(date: &Time, hours: &Option<i64>) -> bool {
    if let Some(h) = hours {
        Utc::now() - chrono::Duration::hours(*h) < date.0
    } else {
        true
    }
}

fn merge<T>(a: &mut Option<T>, b: Option<T>, new: bool) {
    if b.is_none() || !new && !a.is_none() {
        return;
    }
    *a = b;
}
