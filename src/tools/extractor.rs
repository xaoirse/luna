use async_trait::async_trait;
use std::convert::From;
use std::future::Future;

use crate::model::mongo::Host;

pub trait Pattern {
    fn pattern() -> &'static str;
}
impl Pattern for Host {
    fn pattern() -> &'static str {
        r"((?:[0-9\-a-z]+\.)+[a-z]+)(?:$|[\D\W]+)((?:[0-9]{1,3}\.){3}[0-9]{1,3})?(?:$|[\D\W\s])"
    }
}
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
#[async_trait]
pub trait Extractor {
    fn extract<T>(self, pattern: &str) -> Vec<T>
    where
        T: for<'a> From<regex::Captures<'a>>;

    async fn extract_for_each<T, Fut>(
        &self,
        f: impl FnOnce(T) -> Fut + Send + Copy + 'async_trait,
    ) -> &Self
    where
        T: for<'a> From<regex::Captures<'a>> + Pattern + Send,
        Fut: Future<Output = ()> + Send;
}

#[async_trait]
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

    async fn extract_for_each<T, Fut>(
        &self,
        f: impl FnOnce(T) -> Fut + Send + Copy + 'async_trait,
    ) -> &Self
    where
        T: for<'a> From<regex::Captures<'a>> + Pattern + Send,
        Fut: Future<Output = ()> + Send,
    {
        let re = regex::RegexBuilder::new(T::pattern())
            .multi_line(true)
            .build()
            .unwrap();

        for t in re.captures_iter(self).map(|c| c.into()) {
            f(t).await;
        }
        self
    }
}
