pub mod host;
pub mod job;
pub mod luna;
pub mod program;
pub mod scope;
pub mod service;
pub mod sub;
pub mod tech;
pub mod url;

pub use host::Host;
pub use luna::Luna;
pub use program::Program;
pub use scope::Scope;
pub use service::Service;
pub use sub::Sub;
pub use tech::Tech;
pub use url::Url;

pub use crate::alert::Alert;
pub use crate::cmd::run::*;

pub trait Model {
    fn new() -> Self;
    fn same_bucket(b: &mut Self, a: &mut Self) -> bool;
    fn is_same(&self, find: &Filter) -> bool;
}

mod utc_rfc2822 {

    // https://serde.rs/custom-date-format.html

    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(date) = date {
            let s = date.to_rfc2822();
            serializer.serialize_str(&s)
        } else {
            let s = String::new();
            serializer.serialize_str(&s)
        }
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(None);
        }
        match DateTime::parse_from_rfc2822(&s) {
            Ok(date) => Ok(Some(date.with_timezone(&Utc::now().timezone()))),
            Err(_) => Err(serde::de::Error::custom("Parse Error")),
        }
    }
}

fn merge<T>(a: &mut Option<T>, b: &mut Option<T>, new: bool) {
    if !b.is_some() || !new && !a.is_none() {
        return;
    }
    *a = b.take();
}

fn has(text: &Option<String>, pat: &Option<String>) -> bool {
    match (pat, text) {
        (Some(pat), Some(text)) => text.to_lowercase().contains(pat),
        (None, _) => true,
        _ => false,
    }
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
    fn test_has() {
        assert!(has(&Some("abcd".to_string()), &Some("cd".to_string())));
        assert!(!has(&Some("abc".to_string()), &Some("ef".to_string())));
        assert!(has(&Some("abcd".to_string()), &None));
        assert!(!has(&None, &Some("abcd".to_string())));
        assert!(has(&None, &None));
    }
}
