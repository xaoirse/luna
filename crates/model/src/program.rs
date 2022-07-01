use super::*;

#[derive(Debug, Parser, Deserialize, Serialize, Default)]
pub struct Program {
    pub name: String,
    #[clap(long)]
    pub platform: Option<String>,
    #[clap(long)]
    pub handle: Option<String>,
    #[clap(long = "type")]
    pub typ: Option<String>,
    #[clap(long)]
    pub url: Option<String>,
    #[clap(long)]
    pub bounty: Option<String>,
    #[clap(long)]
    pub state: Option<String>,

    #[clap(long, short, multiple_values = true)]
    pub assets: Vec<Asset>,

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
        let new = self.start < other.start;

        merge(&mut self.platform, other.platform, new);
        merge(&mut self.handle, other.handle, new);
        merge(&mut self.typ, other.typ, new);
        merge(&mut self.url, other.url, new);
        merge(&mut self.bounty, other.bounty, new);
        merge(&mut self.state, other.state, new);

        self.start = self.start.min(other.start);

        for asset in other.assets {
            self.insert_asset(asset);
        }
    }

    pub fn assets_search(&self, asset: &Asset) -> Result<usize, usize> {
        if let AssetName::Url(s_req) = &asset.name {
            // Path size
            let s = s_req.url.path_segments().map_or(0, |s| {
                s.filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                    .count()
            });

            // Index of 's' sized
            let a = self.assets.partition_point(|a| {
                if let AssetName::Url(req) = &a.name {
                    let size = req.url.path_segments().map_or(0, |s| {
                        s.filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                            .count()
                    });
                    req.url[..url::Position::BeforePath] < s_req.url[..url::Position::BeforePath]
                        || (req.url[..url::Position::BeforePath]
                            == s_req.url[..url::Position::BeforePath]
                            && size < s)
                } else {
                    a < asset
                }
            });

            // Index of 's' sized
            let b = self.assets.partition_point(|a| {
                if let AssetName::Url(req) = &a.name {
                    let size = req.url.path_segments().map_or(0, |s| {
                        s.filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                            .count()
                    });
                    req.url[..url::Position::BeforePath] < s_req.url[..url::Position::BeforePath]
                        || (req.url[..url::Position::BeforePath]
                            == s_req.url[..url::Position::BeforePath]
                            && size < s + 1)
                } else {
                    a < asset
                }
            });

            // find x in a..b
            for x in a..b {
                if &self.assets[x] == asset {
                    return Ok(x);
                }
            }
        }

        self.assets.binary_search(asset)
    }

    pub fn insert_asset(&mut self, asset: Asset) -> u8 {
        match self.assets_search(&asset) {
            Ok(i) => {
                self.assets[i].merge(asset);
                0
            }
            Err(i) => {
                let ret = match asset.name {
                    AssetName::Url(ref req) => {
                        if let Some(host) = req.url.host() {
                            if let Ok(host) = Asset::from_str(&host.to_string()) {
                                self.insert_asset(host)
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    }

                    AssetName::Subdomain(_) => {
                        if let Some(domain) = asset.name.domain() {
                            let domain = Asset {
                                name: domain,
                                tags: vec![],
                                start: time::Time::default(),
                            };

                            self.insert_asset(domain)
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
                self.assets.insert(i + ret as usize, asset);
                1 + ret
            }
        }
    }

    // Aggregate CIDRs
    pub fn aggregate(&mut self) {
        let nets = self
            .assets
            .iter()
            .filter_map(|a| match a.name {
                AssetName::Cidr(c) => Some(c),
                _ => None,
            })
            .collect();

        self.assets
            .retain(|a| !matches!(a.name, AssetName::Cidr(_)));

        let nets = IpNet::aggregate(&nets);

        for n in nets {
            self.assets.push(Asset {
                name: AssetName::Cidr(n),
                tags: vec![],
                start: time::Time::default(),
            });
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
            .filter(|a| filter.start.map_or(true, |t| t < a.start))
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
    Assets:   {}
    Domains:  {}
    CIDRs:    {}
    Subs:     {}
    URLs:     {}
    Tags:     {}
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
                self.assets.iter().flat_map(|a| &a.tags).count(),
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
    Assets:   {}
    Domains:  [{}{}
    CIDRs:    [{}{}
    Subs:     {}
    URLs:     {}
    Tags:     {}
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
                self.assets(Field::Cidr, &Filter::default())
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(0)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.assets(Field::Cidr, &Filter::default()).is_empty() {
                    "]"
                } else {
                    "\n    ]"
                },
                self.assets(Field::Sub, &Filter::default()).len(),
                self.assets(Field::Url, &Filter::default()).len(),
                self.assets.iter().flat_map(|a| &a.tags).count(),
                self.start
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
            ),
            _ => format!("{:#?}", self),
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn partition_point_test() {
        use super::*;

        let r = Request::from_str("http://w.com/f/c").unwrap();
        let assets = vec![
            Asset::from_str("http://w.com/a").unwrap(),
            Asset::from_str("http://w.com/a/b/").unwrap(),
            Asset::from_str("http://w.com/a/b/c").unwrap(),
            Asset::from_str("http://w.com/a/b/c/d").unwrap(),
            Asset::from_str("http://x.com/a/").unwrap(),
            Asset::from_str("http://x.com/a/b").unwrap(),
            Asset::from_str("http://x.com/a/b/c").unwrap(),
            Asset::from_str("http://y.com/a/").unwrap(),
            Asset::from_str("http://y.com/a/b").unwrap(),
            Asset::from_str("http://y.com/a/b/c").unwrap(),
            Asset::from_str("http://y.com/a/b/c/d/e").unwrap(),
            Asset::from_str("http://y.com/a/b/c/d/e").unwrap(),
            Asset::from_str("http://y.com/a/b/c/d/e").unwrap(),
        ];

        let a = assets.partition_point(|a| {
            if let AssetName::Url(req) = &a.name {
                let size = req.url.path_segments().map_or(0, |s| {
                    s.filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                        .count()
                });

                req.url[..url::Position::BeforePath] < r.url[..url::Position::BeforePath]
                    || (req.url[..url::Position::BeforePath] == r.url[..url::Position::BeforePath]
                        && size < 2)
            } else {
                true
            }
        });

        println!("{a}");
    }

    #[test]
    fn partition_point_test_1() {
        use super::*;

        let r = Asset::from_str("d.com").unwrap();
        let assets = vec![
            Asset::from_str("a.com").unwrap(),
            Asset::from_str("b.com").unwrap(),
            Asset::from_str("c.com").unwrap(),
            Asset::from_str("e.com").unwrap(),
            Asset::from_str("http://w.com/a").unwrap(),
            Asset::from_str("http://w.com/a/b/").unwrap(),
            Asset::from_str("http://w.com/a/b/c").unwrap(),
            Asset::from_str("http://w.com/a/b/c/d").unwrap(),
            Asset::from_str("http://x.com/a/").unwrap(),
            Asset::from_str("http://x.com/a/b").unwrap(),
            Asset::from_str("http://x.com/a/b/c").unwrap(),
            Asset::from_str("http://x.com/a/b/c").unwrap(),
            Asset::from_str("http://x.com/a/b/c").unwrap(),
            Asset::from_str("http://x.com/a/b/c/d/e").unwrap(),
        ];

        let a = assets.partition_point(|a| {
            if let (AssetName::Url(req), AssetName::Url(r)) = (&a.name, &r.name) {
                let size = req.url.path_segments().map_or(0, |s| {
                    s.filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                        .count()
                });

                req.url[..url::Position::BeforePath] < r.url[..url::Position::BeforePath]
                    || (req.url[..url::Position::BeforePath] == r.url[..url::Position::BeforePath]
                        && size < 2)
            } else {
                a < &r
            }
        });

        assert_eq!(a, 3);
    }

    #[test]
    fn assets_search_test_1() {
        use super::*;

        assert!(
            Asset::from_str("http://x.com/b/a").unwrap()
                == Asset::from_str("http://x.com/g/a").unwrap()
        );

        let assets = vec![
            Asset::from_str("http://x.com/a").unwrap(),
            Asset::from_str("http://x.com/a/b").unwrap(),
            Asset::from_str("http://x.com/b/a").unwrap(),
            Asset::from_str("http://x.com/c/d").unwrap(),
            Asset::from_str("http://x.com/a/a/c").unwrap(),
            Asset::from_str("http://x.com/g/a").unwrap(),
            Asset::from_str("http://x.com/g/a/b").unwrap(),
            Asset::from_str("http://x.com/g/a/g/c").unwrap(),
            Asset::from_str("http://x.com/g/a/h/h").unwrap(),
            Asset::from_str("http://x.com/g/a/h/h/f").unwrap(),
            Asset::from_str("http://x.com/g/a/h/h/f/k").unwrap(),
        ];

        let p1 = Program {
            name: "p1".to_string(),
            assets,
            ..Default::default()
        };
        let mut p2 = Program {
            name: "p2".to_string(),
            ..Default::default()
        };

        p2.merge(p1);

        for a in &p2.assets {
            println!("{}", a.name);
        }

        assert_eq!(
            p2.assets_search(&Asset::from_str("http://x.com/a/").unwrap()),
            Ok(1)
        );
        assert_eq!(
            p2.assets_search(&Asset::from_str("http://x.com/g/a/").unwrap()),
            Ok(3)
        );
        assert_eq!(
            p2.assets_search(&Asset::from_str("http://x.com/g/a/b").unwrap()),
            Ok(6)
        );
    }

    #[test]
    fn assets_search_test_2() {
        use super::*;

        let assets = vec![
            Asset::from_str("http://x.com/a").unwrap(),
            Asset::from_str("http://x.com/a/b/").unwrap(),
            Asset::from_str("http://x.com/a/b").unwrap(),
            Asset::from_str("http://x.com/a/b/c").unwrap(),
            Asset::from_str("http://x.com/a/b/c").unwrap(),
            Asset::from_str("http://x.com/a/b/c").unwrap(),
            Asset::from_str("http://x.com/a/b/c/d/e").unwrap(),
        ];

        let p1 = Program {
            name: "p1".to_string(),
            assets,
            ..Default::default()
        };

        assert_eq!(
            p1.assets_search(&Asset::from_str("http://x.com/a/b/").unwrap()),
            Ok(1)
        );
        assert_eq!(
            p1.assets_search(&Asset::from_str("http://x.com/a/b/c/d/e").unwrap()),
            Ok(6)
        );

        assert_eq!(
            p1.assets_search(&Asset::from_str("http://x.com/a/b/c/d/").unwrap()),
            Err(6)
        );
    }

    #[test]
    fn asset_ord_0() {
        use super::*;

        let assets = vec![
            Asset::from_str("http://sub.x.com/ae/ar").unwrap(),
            Asset::from_str("http://sub.x.com/api/history-media/").unwrap(),
            Asset::from_str("http://sub.x.com/api/search/?q=ambasador").unwrap(),
            Asset::from_str("http://sub.x.com/api/loc/?loc=16&co=de&pi=6&lang=de").unwrap(),
            Asset::from_str("http://sub.x.com/api/Wayin/?feed=t&format=json&limit=8").unwrap(),
            Asset::from_str("http://sub.x.com/at/en").unwrap(),
            Asset::from_str("http://sub.x.com/be/nl").unwrap(),
        ];

        let p1 = Program {
            name: "p1".to_string(),
            assets,
            ..Default::default()
        };

        let mut p2 = Program {
            name: "p2".to_string(),
            ..Default::default()
        };

        p2.merge(p1);

        for a in &p2.assets {
            println!("{}", a.name);
        }

        assert_eq!(
            Asset::from_str("http://sub.x.com/api/history-media/?format=json&loc=16&lang=de&pi=6&q=ambasador&feed=t&co=de").unwrap(),
            Asset::from_str("http://sub.x.com/api/history-media/?loc=16&pi=6&feed=t&lang=de&format=json&q=ambasador&co=de").unwrap(),
        );

        assert_eq!(
            p2.assets,
            vec![
                Asset::from_str("x.com").unwrap(),
                Asset::from_str("sub.x.com").unwrap(),
                Asset::from_str("http://sub.x.com/ae/ar").unwrap(),
                Asset::from_str("http://sub.x.com/api/Wayin/?feed=t&format=json&limit=8").unwrap(),
                Asset::from_str("http://sub.x.com/api/history-media/").unwrap(),
                Asset::from_str("http://sub.x.com/api/loc/?loc=16&co=de&pi=6&lang=de").unwrap(),
                Asset::from_str("http://sub.x.com/api/search/?q=ambasador").unwrap(),
                Asset::from_str("http://sub.x.com/at/en").unwrap(),
                Asset::from_str("http://sub.x.com/be/nl").unwrap(),
            ],
        );
    }

    #[test]
    fn asset_ord_1() {
        use super::*;

        let assets = vec![
            Asset::from_str("https://x.com/pregnancy/pregnancy-health/%20/lgbtq?").unwrap(),
            Asset::from_str("https://x.com/pregnancy/pregnancy-health/complications/6-ectopic-pregnancy-symptoms?").unwrap(),
            Asset::from_str("https://x.com/pregnancy/pregnancy-lifestyle/%20/being-a-mom").unwrap(),
            Asset::from_str("https://x.com/pregnancy/week-by-week/%20/being-a-mom?").unwrap(),
        ];

        let p1 = Program {
            name: "p1".to_string(),
            assets,
            ..Default::default()
        };

        let mut p2 = Program {
            name: "p2".to_string(),
            ..Default::default()
        };

        p2.merge(p1);

        for a in &p2.assets {
            println!("{}", a.name);
        }

        assert_eq!(
            p2.assets,
            vec![
                Asset::from_str("x.com").unwrap(),
                Asset::from_str("https://x.com/pregnancy/pregnancy-health/%20/lgbtq?").unwrap(),
                Asset::from_str("https://x.com/pregnancy/pregnancy-health/complications/6-ectopic-pregnancy-symptoms?").unwrap(),
                // Asset::from_str("https://x.com/pregnancy/pregnancy-lifestyle/%20/being-a-mom").unwrap(),
                Asset::from_str("https://x.com/pregnancy/week-by-week/%20/being-a-mom?").unwrap(),
            ],
        );
    }
}
