use std::collections::HashSet;

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
    #[serde(with = "serde_time")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "serde_time")]
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

        let mut i = b.tags.len();
        while i > 0 {
            i -= 1;

            let b = b.tags.swap_remove(i);
            if let Some(a) = a.tags.par_iter_mut().find_any(|a| &&b == a) {
                Tag::same(b, a);
            } else {
                a.tags.push(b);
            }
        }
    }
    fn dedup(&mut self) {
        dedup(&mut self.tags);
    }
    fn is_empty(&self) -> bool {
        self.url == urlib::Url::parse("http://default.url").unwrap()
    }
    fn no_name(&self) -> bool {
        self.url == urlib::Url::parse("http://default.url").unwrap()
    }
}

impl Url {
    pub fn same(mut b: Self, a: &mut Self) {
        let new = a.update < b.update;

        merge(&mut a.title, &mut b.title, new);
        merge(&mut a.status_code, &mut b.status_code, new);
        merge(&mut a.response, &mut b.response, new);

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        for b in b.tags {
            if let Some(a) = a.tags.par_iter_mut().find_any(|a| &&b == a) {
                Tag::same(b, a);
            } else {
                a.tags.push(b);
            }
        }
    }

    pub fn clear(&mut self) {
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
                    .map(|s| format!("\n        {}", s.stringify(2)))
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
        other.url[..urlib::Position::BeforePath] == self.url[..urlib::Position::BeforePath]
            && (self.url.path() == other.url.path()
                || match (other.url.path_segments(), self.url.path_segments()) {
                    (Some(a), Some(b)) => {
                        let a: HashSet<&str> = a
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();
                        let b: HashSet<&str> = b
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();

                        let c = &a - &b;
                        let d = &b - &a;

                        c.len() < 2 && d.len() < 2
                    }
                    (None, None) => true,
                    _ => false,
                })
            && {
                if self.url.path().ends_with(".css") && other.url.path().ends_with(".css") {
                    return true;
                }
                if self.url.path().ends_with(".js") && other.url.path().ends_with(".js") {
                    return true;
                }
                true
            }
            && {
                let a: HashSet<_> = other
                    .url
                    .query_pairs()
                    .map(|(k, _)| k)
                    .filter(|s| !s.is_empty())
                    .collect();
                let b: HashSet<_> = self
                    .url
                    .query_pairs()
                    .map(|(k, _)| k)
                    .filter(|s| !s.is_empty())
                    .collect();

                a == b
            }
    }
}

impl Eq for Url {}

mod test {

    #[test]
    fn test_eq() {
        use super::Url;
        use std::str::FromStr;

        assert_eq!(
            Url::from_str("https://app.lexoffice.de/voucher/settings/template/popover/voucher/settings/text/text/lib/css/libs.css?v=c644c27a4").unwrap(),
            Url::from_str("https://app.lexoffice.de/voucher/settings/template/popover/voucher/settings/voucher/text/settings/voucher/settings/lib/css/libs.css?v=c644c27a4").unwrap()
        );

        assert_eq!(
            Url::from_str("https://memoryleaks.ir/tag/%d8%a2%d8%b3%db%8c%d8%a8-%d9%be%d8%b0%db%8c%d8%b1%db%8c-%d8%af%d8%b1-%d8%b1%d8%a8%d8%a7%d8%aa-%d9%87%d8%a7%db%8c-%d8%aa%d9%84%da%af%d8%b1%d8%a7%d9%85/feed/").unwrap(),
            Url::from_str("https://memoryleaks.ir/tag/%d8%af%d9%88%d8%b1-%d8%b2%d8%af%d9%86-%d9%85%da%a9%d8%a7%d9%86%db%8c%d8%b2%d9%85-%d8%aa%d8%b4%d8%ae%db%8c%d8%b5-%d8%af%d8%b3%d8%aa%da%af%d8%a7%d9%87-%d8%b1%d9%88%d8%aa-%d8%b4%d8%af%d9%87/feed/").unwrap()
        );

        assert_eq!(
            Url::from_str("http://aktuelles.haufe.de/eagle_kp_webapp/login/login.action;jsessionid=2acc8603184240a22e4fa5fe3525").unwrap(),
            Url::from_str("http://aktuelles.haufe.de/eagle_kp_webapp/login/login.action;jsessionid=2b0107dc883c28019e877799d07e").unwrap()
        );

        assert_eq!(
            Url::from_str("https://memoryleaks.ir/category/%d8%a7%d8%b1%d8%aa%d9%82%d8%a7%d8%b9-%d8%af%d8%b3%d8%aa%d8%b1%d8%b3%db%8c/feed/").unwrap(),
            Url::from_str("https://memoryleaks.ir/category/%d8%a7%d9%85%d9%86%db%8c%d8%aa-%d8%a7%d9%86%d8%af%d8%b1%d9%88%db%8c%d8%af/feed/").unwrap()
        );

        assert_ne!(
            Url::from_str("https://a.com").unwrap(),
            Url::from_str("https://b.com").unwrap()
        );
        assert_eq!(
            Url::from_str("https://a.com/").unwrap(),
            Url::from_str("https://a.com").unwrap()
        );

        assert_eq!(
            Url::from_str("https://a.com/a/b/c/").unwrap(),
            Url::from_str("https://a.com/a/d/c").unwrap()
        );
        assert_ne!(
            Url::from_str("https://a.com/a/b/c/").unwrap(),
            Url::from_str("https://a.com/a/d/e").unwrap()
        );

        assert_ne!(
            Url::from_str("https://a.com/a/23/b").unwrap(),
            Url::from_str("https://b.com/b/a/23/a/").unwrap()
        );
    }

    #[test]
    fn dedup() {
        use super::*;
        use std::str::FromStr;

        let mut arr = vec![
            Sub {
                urls: vec![Url::from_str("https://a.com/a/a.html").unwrap()],
                ..Default::default()
            },
            Sub {
                urls: vec![Url::from_str("https://a.com/a/b.html").unwrap()],
                ..Default::default()
            },
        ];
        dedup(&mut arr);

        assert_eq!(
            arr,
            vec![Sub {
                urls: vec![Url::from_str("https://a.com/a/a.html").unwrap()],
                ..Default::default()
            },]
        );

        let mut arr = vec![
            Sub {
                urls: vec![Url::from_str("https://a.com").unwrap()],
                ..Default::default()
            },
            Sub {
                urls: vec![Url::from_str("https://a.com/a/b?mia=love").unwrap()],
                ..Default::default()
            },
        ];
        dedup(&mut arr);
        assert_ne!(
            arr,
            vec![Sub {
                urls: vec![Url::from_str("https://a.com/a/b?mia=love").unwrap()],
                ..Default::default()
            }],
        );

        let mut arr = vec![
                    Sub {
                        urls: vec![Url::from_str("https://memoryleaks.ir/category/%D9%88%DB%8C%D8%AF%D8%A6%D9%88-%D8%A2%D9%85%D9%88%D8%B2%D8%B4%DB%8C/page/2/").unwrap(),],
                        ..Default::default()
                    },
                    Sub {
                        urls: vec![Url::from_str("https://memoryleaks.ir/category/%d8%a7%d8%b1%d8%aa%d9%82%d8%a7%d8%b9-%d8%af%d8%b3%d8%aa%d8%b1%d8%b3%db%8c/feed/").unwrap(),],
                        ..Default::default()
                    },
                    Sub {
                        urls: vec![Url::from_str("https://memoryleaks.ir/category/%d9%88%db%8c%d8%af%d8%a6%d9%88-%d8%a2%d9%85%d9%88%d8%b2%d8%b4%db%8c/page/2/").unwrap(),],
                        ..Default::default()
                    },
                    Sub {
                        urls: vec![Url::from_str("https://memoryleaks.ir/category/%d8%a7%d9%85%d9%86%db%8c%d8%aa-%d8%a7%d9%86%d8%af%d8%b1%d9%88%db%8c%d8%af/feed/").unwrap()],
                        ..Default::default()
                    }
        ];
        dedup(&mut arr);

        assert_eq!(
            arr,
            vec![Sub {
                        urls: vec![Url::from_str("https://memoryleaks.ir/category/%d9%88%db%8c%d8%af%d8%a6%d9%88-%d8%a2%d9%85%d9%88%d8%b2%d8%b4%db%8c/page/2/").unwrap(),],
                        ..Default::default()
                    },
                    Sub {
                        urls: vec![Url::from_str("https://memoryleaks.ir/category/%d8%a7%d8%b1%d8%aa%d9%82%d8%a7%d8%b9-%d8%af%d8%b3%d8%aa%d8%b1%d8%b3%db%8c/feed/").unwrap(),],
                        ..Default::default()
                    }
            ],
        );
    }
}
