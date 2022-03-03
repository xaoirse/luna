use super::*;
use ::url as urlib;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Parser, Clone)]
pub struct Url {
    #[clap(long)]
    pub url: urlib::Url,

    #[clap(long)]
    pub title: Option<String>,

    #[clap(long)]
    pub status_code: Option<String>,

    #[clap(long, short)]
    pub response: Option<String>,

    #[clap(long)]
    pub tags: Vec<Tag>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
}

impl Dedup for Url {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        let new = a.update < b.update;

        merge(&mut a.title, &mut b.title, new);
        merge(&mut a.status_code, &mut b.status_code, new);
        merge(&mut a.response, &mut b.response, new);

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.tags.append(&mut b.tags);
    }
    fn is_empty(&self) -> bool {
        self.url == urlib::Url::parse("http://default.url").unwrap()
    }
}

impl Url {
    pub fn empty(&mut self) {
        self.url = urlib::Url::parse("http://default.url").unwrap();
    }

    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        self.url.to_string().contains_opt(&filter.url)
            && self.title.contains_opt(&filter.title)
            && self.response.contains_opt(&filter.response)
            && self.status_code.contains_opt(&filter.status_code)
            && (!date
                || (check_date(&self.update, &filter.updated_at)
                    && check_date(&self.start, &filter.started_at)))
            && (filter.tag_is_none() || self.tags.par_iter().any(|t| t.matches(filter, false)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.url.to_string(),
            1 => format!(
                "{} [{}]",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
            ),
            2 => format!(
                "{} [{}] [{}]",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
                self.title.as_ref().map_or("", |s| s),
            ),
            3 => format!(
                "{} [{}]
    Title: {}
    Response length: {}
    Tags: {}
    Update: {}
    Start: {}
    ",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
                self.title.as_ref().map_or("", |s| s),
                self.response
                    .as_ref()
                    .map_or("n".to_string(), |s| s.len().to_string()),
                self.tags.len(),
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            4 => format!(
                "{} [{}]
    Title: {}
    Response length: {}
    Tags: [{}{}
    Update: {}
    Start: {}
    ",
                self.url,
                self.status_code.as_ref().map_or("", |s| s),
                self.title.as_ref().map_or("", |s| s),
                self.response
                    .as_ref()
                    .map_or("n".to_string(), |s| s.len().to_string()),
                self.tags
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(1)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.tags.is_empty() { "]" } else { "\n    ]" },
                self.update.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
                self.start.map_or("".to_string(), |s| s
                    .with_timezone(&chrono::Local::now().timezone())
                    .to_rfc2822()),
            ),
            _ => format!("{:#?}", self),
        }
    }

    pub fn sub_asset(&self) -> Option<&str> {
        self.url.host_str()
    }
}

impl Default for Url {
    fn default() -> Self {
        Self {
            url: urlib::Url::parse("http://default.url").unwrap(),
            title: None,
            status_code: None,
            response: None,
            tags: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
        }
    }
}

impl std::str::FromStr for Url {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Url {
            url: urlib::Url::parse(s)?,
            ..Default::default()
        })
    }
}
impl Ord for Url {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.url.cmp(&self.url)
    }
}

impl PartialOrd for Url {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Url {
    fn eq(&self, other: &Self) -> bool {
        other.url.as_str().contains(&self.url.as_str())
            || self.url.as_str().contains(&other.url.as_str())
            || {
                let aa = other.url.path_segments();
                let bb = self.url.path_segments();

                other.url[..urlib::Position::BeforePath] == self.url[..urlib::Position::BeforePath]
                    && {
                        let mut a = other.url.query_pairs().map(|(k, _)| k).collect::<Vec<_>>();
                        let mut b = self.url.query_pairs().map(|(k, _)| k).collect::<Vec<_>>();
                        a.sort();
                        b.sort();
                        a == b
                    }
                    && match (aa, bb) {
                        (Some(a), Some(b)) => {
                            let n = a
                                .zip(b)
                                .fold(0, |init, (a, b)| if a == b { init } else { init + 1 });
                            n < 2
                        }
                        _ => false,
                    }
            }
    }
}

impl Eq for Url {}

mod test {

    #[test]
    fn test_eq() {
        use super::Url;
        use std::str::FromStr;

        assert_ne!(
            Url::from_str("https://a.com").unwrap(),
            Url::from_str("https://b.com").unwrap()
        );
        assert_eq!(
            Url::from_str("https://a.com/").unwrap(),
            Url::from_str("https://a.com").unwrap()
        );
        assert_eq!(
            Url::from_str("https://a.com/a/b?mia=love").unwrap(),
            Url::from_str("https://a.com/a").unwrap()
        );
        assert_eq!(
            Url::from_str("https://a.com/a/b/c/").unwrap(),
            Url::from_str("https://a.com/a/d/c").unwrap()
        );
        assert_ne!(
            Url::from_str("https://a.com/a/b/c/").unwrap(),
            Url::from_str("https://a.com/a/d/e").unwrap()
        );
        assert_eq!(
            Url::from_str("https://a.com/a/").unwrap(),
            Url::from_str("https://a.com/a/d/e").unwrap()
        );
    }

    #[test]
    fn dedup() {
        use super::*;
        use std::str::FromStr;

        let mut arr = vec![
            Url::from_str("https://a.com/a/a.html").unwrap(),
            Url::from_str("https://a.com/a/b.html").unwrap(),
        ];
        arr.sort();
        arr.dedup();
        assert_eq!(arr, vec![Url::from_str("https://a.com/a/b.html").unwrap(),]);

        let mut arr = vec![
            Url::from_str("https://a.com").unwrap(),
            Url::from_str("https://b.com").unwrap(),
            Url::from_str("https://a.com/").unwrap(),
            Url::from_str("https://a.com").unwrap(),
            Url::from_str("https://a.com/a/b?mia=love").unwrap(),
            Url::from_str("https://a.com/a").unwrap(),
            Url::from_str("https://a.com/a/b/c/").unwrap(),
            Url::from_str("https://a.com/a/d/c").unwrap(),
            Url::from_str("https://a.com/a/b/c/").unwrap(),
            Url::from_str("https://a.com/a/d/e").unwrap(),
            Url::from_str("https://a.com/a/").unwrap(),
            Url::from_str("https://a.com/a/d/e").unwrap(),
        ];
        arr.sort();
        arr.dedup();
        assert_eq!(
            arr,
            vec![
                Url::from_str("https://b.com").unwrap(),
                Url::from_str("https://a.com/a/d/e").unwrap(),
                Url::from_str("https://a.com/a/b?mia=love").unwrap(),
                Url::from_str("https://a.com/a/b/c/").unwrap(),
            ]
        );

        let mut arr = vec![
            Url::from_str("https://b.com").unwrap(),
            Url::from_str("https://a.com/").unwrap(),
            Url::from_str("https://a.com/a/b?mia=love").unwrap(),
            Url::from_str("https://a.com/a").unwrap(),
            Url::from_str("https://a.com/a/d/c").unwrap(),
            Url::from_str("https://a.com/a/b/c/").unwrap(),
            Url::from_str("https://a.com/a/d/e").unwrap(),
            Url::from_str("https://a.com/a/b/e").unwrap(),
            Url::from_str("https://a.com").unwrap(),
            Url::from_str("https://a.com/a/").unwrap(),
            Url::from_str("https://a.com/a/d/e").unwrap(),
            Url::from_str("https://a.com").unwrap(),
            Url::from_str("https://a.com/a/b/c/").unwrap(),
        ];
        arr.sort();
        arr.dedup();
        assert_eq!(
            arr,
            vec![
                Url::from_str("https://b.com").unwrap(),
                Url::from_str("https://a.com/a/d/e").unwrap(),
                Url::from_str("https://a.com/a/b?mia=love").unwrap(),
                Url::from_str("https://a.com/a/b/c/").unwrap(),
            ]
        );
    }
}
