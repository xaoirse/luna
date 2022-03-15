use chrono::Utc;
use rayon::prelude::*;

pub mod asset;
pub mod filter;
pub mod luna;
pub mod program;
pub mod run;
pub mod script;
pub mod serde_cidr;
pub mod tag;
pub mod time;

pub use asset::*;
pub use filter::Filter;
pub use luna::Luna;
pub use program::Program;
pub use run::*;
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
