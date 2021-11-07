use crate::model::mongo::Host;
use std::convert::From;

impl<'t> From<regex::Captures<'t>> for Host {
    fn from(cap: regex::Captures<'t>) -> Self {
        Host {
            sub: cap
                .get(1)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            ip: cap
                .get(2)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            services: vec![],
            update: None,
        }
    }
}

// https://doc.rust-lang.org/nomicon/hrtb.html
pub trait Extractor {
    fn extract<T>(self, pattern: &str) -> Vec<T>
    where
        T: for<'a> From<regex::Captures<'a>>;
}
impl Extractor for String {
    fn extract<T>(self, pattern: &str) -> Vec<T>
    where
        T: for<'a> From<regex::Captures<'a>>,
    {
        let re = regex::RegexBuilder::new(pattern)
            .multi_line(true)
            .build()
            .unwrap();

        re.captures_iter(&self).map(|c| c.into()).collect()
    }
}
