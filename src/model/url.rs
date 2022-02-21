use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt, Clone)]
pub struct Url {
    #[structopt(long)]
    pub url: String,

    #[structopt(long)]
    pub title: Option<String>,

    #[structopt(long)]
    pub status_code: Option<String>,

    #[structopt(long, short)]
    pub response: Option<String>,

    #[structopt(long)]
    pub techs: Vec<Tech>,

    #[structopt(long)]
    pub tags: Vec<Tag>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[structopt(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,
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

        a.techs.append(&mut b.techs);
        a.tags.append(&mut b.tags);
    }
    fn dedup(&mut self, term: Arc<AtomicBool>) {
        dedup(&mut self.techs, term.clone());
        dedup(&mut self.tags, term);
    }
}

impl Url {
    pub fn matches(&self, filter: &FilterRegex) -> bool {
        self.url.contains_opt(&filter.url)
            && self.title.contains_opt(&filter.title)
            && self.response.contains_opt(&filter.response)
            && self.status_code.contains_opt(&filter.status_code)
            && check_date(&self.update, &filter.updated_at)
            && check_date(&self.start, &filter.started_at)
            && (filter.tech_is_none() || self.techs.par_iter().any(|t| t.matches(filter)))
            && (filter.tag_is_none() || self.tags.par_iter().any(|t| t.matches(filter)))
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
    Techs: {}
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
                self.techs.len(),
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
    Techs: [{}{}
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
                self.techs
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(1)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.techs.is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
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
            techs: vec![],
            tags: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
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

impl PartialEq for Url {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Eq for Url {}
