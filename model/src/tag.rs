use super::*;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Parser, Clone)]
pub struct Tag {
    #[clap(short, long)]
    pub name: String,

    #[clap(long)]
    pub severity: Option<String>,

    #[clap(long)]
    pub values: Vec<String>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub update: Option<DateTime<Utc>>,

    #[clap(skip)]
    #[serde(with = "utc_rfc2822")]
    pub start: Option<DateTime<Utc>>,

    #[clap(skip)]
    pub dedup: bool,
}

impl Dedup for Tag {
    fn same_bucket(b: &mut Self, a: &mut Self) {
        let new = a.update < b.update;

        if a.name.is_empty() {
            a.name = std::mem::take(&mut b.name);
        }

        a.update = a.update.max(b.update);
        a.start = a.start.min(b.start);

        merge(&mut a.severity, &mut b.severity, new);

        a.values.append(&mut b.values);
        a.dedup = false;
    }
    fn dedup(&mut self, _term: Arc<AtomicBool>) {
        if self.dedup {
            return;
        }
        self.values.par_sort();
        self.values.dedup();
        self.dedup = true;
    }
    fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
}

impl Tag {
    pub fn matches(&self, filter: &FilterRegex, date: bool) -> bool {
        self.name.contains_opt(&filter.tag)
            && self.severity.contains_opt(&filter.tag_severity)
            && (!date
                || (check_date(&self.update, &filter.updated_at)
                    && check_date(&self.start, &filter.started_at)))
            && filter
                .tag_value
                .as_ref()
                .map_or(true, |tv| self.values.iter().any(|v| tv.is_match(v)))
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!(
                "{} [{}]",
                self.name,
                self.severity.as_ref().map_or("", |s| s),
            ),
            2 => format!(
                "{} [{}]
    values:[{}{}
    Update: {}
    Start: {}
    ",
                self.name,
                self.severity.as_ref().map_or("", |s| s),
                self.values
                    .iter()
                    .map(|s| format!("\n        {}", s))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.values.is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
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

impl Default for Tag {
    fn default() -> Self {
        Self {
            name: String::new(),
            severity: None,
            values: vec![],
            update: Some(Utc::now()),
            start: Some(Utc::now()),
            dedup: false,
        }
    }
}

impl std::str::FromStr for Tag {
    type Err = std::str::Utf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tag {
            name: s.to_string(),
            ..Default::default()
        })
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name.to_lowercase() == other.name.to_lowercase()
    }
}

impl Eq for Tag {}