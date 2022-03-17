use super::*;

#[derive(Debug, Parser, Deserialize, Serialize, Default)]
pub struct Program {
    pub name: String,
    #[clap(long)]
    pub platform: Option<String>,
    #[clap(long)]
    pub handle: Option<String>,
    #[clap(long)]
    pub typ: Option<String>,
    #[clap(long)]
    pub url: Option<String>,
    #[clap(long)]
    pub bounty: Option<String>,
    #[clap(long)]
    pub state: Option<String>,

    #[clap(long)]
    pub assets: Vec<Asset>,

    #[clap(skip)]
    pub update: Time,
    #[clap(skip)]
    pub start: Time,
}

impl FromStr for Program {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: s.to_string(),
            ..Default::default()
        })
    }
}

impl Program {
    pub fn merge(&mut self, other: Self) {
        let new = self.update < other.update;

        merge(&mut self.platform, other.platform, new);
        merge(&mut self.handle, other.handle, new);
        merge(&mut self.typ, other.typ, new);
        merge(&mut self.url, other.url, new);
        merge(&mut self.bounty, other.bounty, new);
        merge(&mut self.state, other.state, new);

        self.update = self.update.max(other.update);
        self.start = self.start.min(other.start);

        for asset in other.assets {
            if let Some(self_asset) = self.assets.iter_mut().find(|a| a.name == asset.name) {
                self_asset.merge(asset);
            } else {
                self.assets.push(asset);
            }
        }
    }
    pub fn assets(&self, field: Field, filter: &Filter) -> Vec<&Asset> {
        self.assets
            .iter()
            .filter(|a| {
                matches!(
                    (&a.name, field),
                    (AssetName::Domain(_), Field::Domain)
                        | (AssetName::Subdomain(_), Field::Sub)
                        | (AssetName::Url(_), Field::Url)
                        | (AssetName::Cidr(_), Field::Cidr)
                        | (_, Field::Asset)
                )
            })
            .filter(|a| filter.asset(a))
            .filter(|a| date(&a.update, &filter.update) || date(&a.start, &filter.start))
            .take(filter.n)
            .collect()
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!("{}  {} ", self.name, self.url.as_ref().map_or("", |s| s)),
            2 => format!(
                "{}  {}
    Platform: {}
    Type:     {}
    Handle:   {}
    Bounty:   {}
    State:    {}
    Asset:    {}
    Domains:  {}
    CIDRs:    {}
    Subs:     {}
    URLs:     {}
    Tags:     {}
    Update:   {}
    Start:    {}
    ",
                self.name,
                self.url.as_ref().map_or("", |s| s),
                self.platform.as_ref().map_or("", |s| s),
                self.typ.as_ref().map_or("", |s| s),
                self.handle.as_ref().map_or("", |s| s),
                self.bounty.as_ref().map_or("", |s| s),
                self.state.as_ref().map_or("", |s| s),
                self.assets.len(),
                self.assets(Field::Domain, &Filter::default()).len(),
                self.assets(Field::Cidr, &Filter::default()).len(),
                self.assets(Field::Sub, &Filter::default()).len(),
                self.assets(Field::Url, &Filter::default()).len(),
                self.assets(Field::Tag, &Filter::default()).len(),
                self.update
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
                self.start
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
            ),
            3 => format!(
                "{}  {}
    Platform: {}
    Type:     {}
    Handle:   {}
    Bounty:   {}
    State:    {}
    Domains: [{}{}
    CIDRs:    {}
    Subs:     {}
    URLs:     {}
    Tags:     {}
    Update:   {}
    Start:    {}
    ",
                self.name,
                self.url.as_ref().map_or("", |s| s),
                self.platform.as_ref().map_or("", |s| s),
                self.typ.as_ref().map_or("", |s| s),
                self.handle.as_ref().map_or("", |s| s),
                self.bounty.as_ref().map_or("", |s| s),
                self.state.as_ref().map_or("", |s| s),
                self.assets(Field::Domain, &Filter::default())
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.assets(Field::Domain, &Filter::default()).is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
                self.assets(Field::Cidr, &Filter::default()).len(),
                self.assets(Field::Sub, &Filter::default()).len(),
                self.assets(Field::Url, &Filter::default()).len(),
                self.assets(Field::Tag, &Filter::default()).len(),
                self.update
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
                self.start
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
            ),
            _ => format!("{:#?}", self),
        }
    }
}
