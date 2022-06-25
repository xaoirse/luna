use chrono::{DateTime, Local, Utc};
use clap::{ArgEnum, Parser};
use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use ipnet::IpNet;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::{
    fmt::{self, Display},
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
    str::FromStr,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use url::Host;

pub mod asset;
pub mod filter;
pub mod luna;
pub mod program;
pub mod request;
pub mod script;
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

fn merge<T>(a: &mut Option<T>, b: Option<T>, new: bool) {
    if b.is_none() || !new && !a.is_none() {
        return;
    }
    *a = b;
}
