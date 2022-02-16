pub mod filter;
pub mod host;
pub mod job;
pub mod luna;
pub mod program;
pub mod scope;
pub mod service;
pub mod sub;
pub mod tech;
pub mod url;
pub mod utc_rfc2822;

pub use crate::cmd::run::*;
pub use filter::{Fields, Filter, FilterRegex, IpCidr};
pub use host::Host;
pub use luna::Luna;
pub use program::Program;
use regex::Regex;
pub use scope::Scope;
pub use scope::ScopeType;
pub use service::Service;
pub use sub::Sub;
pub use tech::Tech;
pub use url::Url;

pub type Errors = Box<dyn std::error::Error + Sync + Send>;

pub trait EqExt {
    fn contains_opt(&self, regex: &Option<Regex>) -> bool;
}
impl EqExt for Option<String> {
    fn contains_opt(&self, regex: &Option<Regex>) -> bool {
        match (self, regex) {
            (Some(text), Some(re)) => re.captures(text).is_some(),
            (_, None) => true,
            _ => false,
        }
    }
}
impl EqExt for String {
    fn contains_opt(&self, regex: &Option<Regex>) -> bool {
        regex
            .as_ref()
            .map_or(true, |re| re.captures(self).is_some())
    }
}

use chrono::{DateTime, Utc};

pub fn check_date(date: &Option<DateTime<Utc>>, days: &Option<i64>) -> bool {
    match (date, days) {
        (Some(date), Some(d)) => &(Utc::now() - chrono::Duration::days(*d)) < date,
        (_, None) => true,
        _ => false,
    }
}

fn merge<T>(a: &mut Option<T>, b: &mut Option<T>, new: bool) {
    if !b.is_some() || !new && !a.is_none() {
        return;
    }
    *a = b.take();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge_0() {
        let mut a = Some("A");
        let mut b = Some("B");
        merge(&mut a, &mut b, true);
        assert_eq!(a, Some("B"));

        let mut a = Some("A");
        let mut b = Some("B");
        merge(&mut a, &mut b, false);
        assert_eq!(a, Some("A"));
    }
    #[test]
    fn test_merge_1() {
        let mut a = Some("A");
        let mut b = None;
        merge(&mut a, &mut b, true);
        assert_eq!(a, Some("A"));

        let mut a = Some("A");
        let mut b = None;
        merge(&mut a, &mut b, false);
        assert_eq!(a, Some("A"));
    }
    #[test]
    fn test_merge_2() {
        let mut a = None;
        let mut b = Some("B");
        merge(&mut a, &mut b, true);
        assert_eq!(a, Some("B"));

        let mut a = None;
        let mut b = Some("B");
        merge(&mut a, &mut b, false);
        assert_eq!(a, Some("B"));
    }
    #[test]
    fn test_merge_3() {
        let mut a: Option<u8> = None;
        let mut b = None;
        merge(&mut a, &mut b, true);
        assert_eq!(a, None);

        merge(&mut a, &mut b, false);
        assert_eq!(a, None);
    }

    #[test]
    fn test_contains_opt() {
        assert!(Some("abcd".to_string()).contains_opt(&Some(regex::Regex::new("ab").unwrap())));
        assert!(!Some("abcd".to_string()).contains_opt(&Some(regex::Regex::new("gf").unwrap())));
        assert!(!None.contains_opt(&Some(regex::Regex::new("gf").unwrap())));
        assert!(None.contains_opt(&None));
    }
}
