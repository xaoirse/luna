use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::From;
use std::future::Future;

use crate::model::mongo::{Host, Sub, SubType, URL};

pub trait Regexer {
    fn regex() -> Regex;
}
lazy_static! {}
impl Regexer for Host {
    fn regex() -> Regex {
        // r"((?:[0-9\-a-z]+\.)+[a-z]+)(?:$|[\D\W]+)((?:[0-9]{1,3}\.){3}[0-9]{1,3})?(?:$|[\D\W\s])"
        static PAT: &str = r"((?:[a-z0-9A-Z]\.)*[a-z0-9-]+\.(?:[a-z0-9]{2,24})+(?:\.co\.(?:[a-z0-9]{2,24})|\.(?:[a-z0-9]{2,24}))*)[\W]*((?:(?:2(?:[0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9])\.){3}(?:(?:2(?:[0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9])))";
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(PAT)
                .multi_line(true)
                .build()
                .unwrap();
        }
        RE.clone()
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

impl Regexer for Sub {
    fn regex() -> Regex {
        // r"((?:[a-z0-9A-Z]\.)*[a-z0-9-]+\.(?:[a-z0-9]{2,24})+(?:\.co\.(?:[a-z0-9]{2,24})|\.(?:[a-z0-9]{2,24}))*)(?:[\W])*((?:(?:2(?:[0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9])\.){3}(?:(?:2(?:[0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9])))(?:$|[\D\W])"
        static PAT: &str = r"((?:[a-z0-9A-Z]\.)*[a-z0-9-]+\.(?:[a-z0-9]{2,24})+(?:\.co\.(?:[a-z0-9]{2,24})|\.(?:[a-z0-9]{2,24}))*)[\W]*((?:(?:2(?:[0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9])\.){3}(?:(?:2(?:[0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9])))?";
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(PAT)
                .multi_line(true)
                .build()
                .unwrap();
        }
        RE.clone()
    }
}
impl<'t> From<regex::Captures<'t>> for Sub {
    fn from(cap: regex::Captures<'t>) -> Self {
        Sub {
            asset: cap
                .get(1)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            scope: "".to_string(),
            host: cap.get(2).map_or(None, |m| Some(m.as_str().to_string())),
            ty: Some(SubType::Domain),
            urls: vec![],
            update: None,
        }
    }
}

//
impl Regexer for URL {
    fn regex() -> Regex {
        static PAT: &str = r"(\w+)://[-a-zA-Z0-9:@;?&=/%\+\.\*!'\(\),\$_\{\}\^~\[\]`#|]+";
        // TODO scheme is in match 1
        lazy_static! {
            static ref RE: Regex = regex::RegexBuilder::new(PAT)
                .multi_line(true)
                .build()
                .unwrap();
        }
        RE.clone()
    }
}
impl<'t> From<regex::Captures<'t>> for URL {
    fn from(cap: regex::Captures<'t>) -> Self {
        URL {
            url: cap
                .get(0)
                .map_or("".to_string(), |m| m.as_str().to_string()),
            sub: "".to_string(),
            title: None,
            status_code: None,
            content_type: None,
            techs: vec![],
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
        T: for<'a> From<regex::Captures<'a>> + Regexer + Send,
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
        T: for<'a> From<regex::Captures<'a>> + Regexer + Send,
        Fut: Future<Output = ()> + Send,
    {
        for t in T::regex().captures_iter(self).map(|c| c.into()) {
            f(t).await;
        }
        self
    }
}
