use super::*;
use ::url as urlib;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Parser, Clone)]
pub struct Url {
    #[clap(long)]
    pub url: String,

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

    #[clap(skip)]
    pub dedup: bool,
}

impl Dedup for Url {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        let new = a.update < b.update;

        if a.url.is_empty() {
            a.url = std::mem::take(&mut b.url);
        }

        merge(&mut a.title, &mut b.title, new);
        merge(&mut a.status_code, &mut b.status_code, new);
        merge(&mut a.response, &mut b.response, new);

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        a.tags.append(&mut b.tags);
        a.dedup = false;
    }
    fn dedup(&mut self, term: Arc<AtomicBool>) {
        if self.dedup {
            return;
        }
        self.dedup = dedup(&mut self.tags, term);
    }
    fn is_empty(&self) -> bool {
        self.url.is_empty()
    }
}

impl Url {
    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        self.url.contains_opt(&filter.url)
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

    pub fn sub_asset(&self) -> Option<String> {
        self.url.split('/').nth(2).map(|s| s.to_string())
    }
}

impl Default for Url {
    fn default() -> Self {
        Self {
            url: String::new(),
            title: None,
            status_code: None,
            response: None,
            tags: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
            dedup: false,
        }
    }
}

impl std::str::FromStr for Url {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Url {
            url: s.to_string(),
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
        other.url.contains(&self.url) || self.url.contains(&other.url) || {
            let a = urlib::Url::parse(&other.url);
            let b = urlib::Url::parse(&self.url);

            match (a, b) {
                (Ok(a), Ok(b)) => {
                    let aa = a.path_segments();
                    let bb = b.path_segments();

                    a[..urlib::Position::BeforePath] == b[..urlib::Position::BeforePath]
                        && {
                            let mut a = a.query_pairs().map(|(k, _)| k).collect::<Vec<_>>();
                            let mut b = b.query_pairs().map(|(k, _)| k).collect::<Vec<_>>();
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
            Url::from_str("https://a.com"),
            Url::from_str("https://b.com")
        );
        assert_eq!(
            Url::from_str("https://a.com/"),
            Url::from_str("https://a.com")
        );
        assert_eq!(
            Url::from_str("https://a.com/a/b?mia=love"),
            Url::from_str("https://a.com/a")
        );
        assert_eq!(
            Url::from_str("https://a.com/a/b/c/"),
            Url::from_str("https://a.com/a/d/c")
        );
        assert_ne!(
            Url::from_str("https://a.com/a/b/c/"),
            Url::from_str("https://a.com/a/d/e")
        );
        assert_eq!(
            Url::from_str("https://a.com/a/"),
            Url::from_str("https://a.com/a/d/e")
        );
    }

    #[test]
    fn test_sort() {
        use super::*;

        use std::str::FromStr;

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
        )
    }
}
