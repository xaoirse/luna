use super::*;

#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
pub struct Asset {
    pub name: AssetName,
    #[clap(long, short, multiple_values = true)]
    pub tags: Vec<Tag>,
    #[clap(skip)]
    pub start: Time,
}

impl FromStr for Asset {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: AssetName::from_str(s)?,
            tags: vec![],
            start: Time::default(),
        })
    }
}

impl Asset {
    pub fn merge(&mut self, other: Self) {
        self.start = self.start.min(other.start);

        for tag in other.tags {
            if let Some(self_tag) = self.tags.iter_mut().find(|t| t.name == tag.name) {
                self_tag.merge(tag);
            } else {
                self.tags.push(tag);
            }
        }

        if let (AssetName::Url(s), AssetName::Url(o)) = (&mut self.name, other.name) {
            let mut s_q = s
                .url
                .query_pairs()
                .filter(|(k, _)| !k.is_empty())
                .collect::<HashMap<_, _>>();
            let o_q = o
                .url
                .query_pairs()
                .filter(|(k, _)| !k.is_empty())
                .collect::<HashMap<_, _>>();

            s_q.extend(o_q);

            let mut query = String::new();
            for m in &s_q {
                if !query.is_empty() {
                    query.push('&');
                }

                query.push_str(&format!("{}", m.0));

                if !m.1.is_empty() {
                    query.push_str(&format!("={}", m.1));
                }
            }

            if !query.is_empty() {
                s.url.set_query(Some(&query));
            }
        }
    }

    pub fn insert_tag(&mut self, tag: Tag) {
        if let Some(p) = self.tag_by_name(&tag.name) {
            p.merge(tag);
        } else {
            self.tags.push(tag);
        }
    }
    pub fn tag_by_name(&mut self, name: &str) -> Option<&mut Tag> {
        self.tags.iter_mut().find(|t| t.name == name)
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => match &self.name {
                AssetName::Url(req) => format!(
                    "{} [{}]",
                    req.url,
                    req.sc.as_ref().unwrap_or(&"".to_string())
                ),
                name => name.to_string(),
            },
            2 => match &self.name {
                AssetName::Url(req) => format!(
                    "{} [{}] [{}]",
                    req.url,
                    req.sc.as_ref().unwrap_or(&"".to_string()),
                    req.title.as_ref().unwrap_or(&"".to_string())
                ),
                name => name.to_string(),
            },
            3 => format!(
                "{}
    Tags:  [{}{}
    ",
                match &self.name {
                    AssetName::Url(req) => format!(
                        "{} [{}] [{}]",
                        req.url,
                        req.sc.as_ref().unwrap_or(&"".to_string()),
                        req.title.as_ref().unwrap_or(&"".to_string())
                    ),
                    name => name.to_string(),
                },
                self.tags
                    .iter()
                    .map(|t| format!("\n        {}", t.stringify(1)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.tags.is_empty() { "]" } else { "\n    ]" },
            ),
            4 => format!(
                "{}
    Tags:  [{}{}
    ",
                self.name,
                self.tags
                    .iter()
                    .map(|t| format!("\n        {}", t.stringify(2)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.tags.is_empty() { "]" } else { "\n    ]" },
            ),
            5 => format!(
                "{}
    Tags:   [{}{}
    Start:  {}
    ",
                self.name,
                self.tags
                    .iter()
                    .map(|t| format!("\n        {}", t.stringify(2)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.tags.is_empty() { "]" } else { "\n    ]" },
                self.start
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
            ),

            _ => format!("{:#?}", self),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AssetName {
    Domain(String),
    Subdomain(Host),
    Url(Request),
    Cidr(IpNet),
}

impl AssetName {
    pub fn domain(&self) -> Option<AssetName> {
        match self {
            AssetName::Subdomain(host) => {
                if let Ok(name) = addr::parse_domain_name(&host.to_string()) {
                    if let Some(root) = name.root() {
                        return Some(AssetName::Domain(root.to_string()));
                    }
                }
                None
            }
            AssetName::Url(request) => {
                if let Some(host) = request.url.host_str() {
                    if let Ok(name) = addr::parse_domain_name(host) {
                        if let Some(root) = name.root() {
                            return Some(AssetName::Domain(root.to_string()));
                        }
                    }
                }
                None
            }
            _ => Some(self.clone()),
        }
    }
}

impl Display for AssetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssetName::Domain(d) => d.to_string(),
                AssetName::Subdomain(s) => s.to_string(),
                AssetName::Url(url) => url.url.to_string(),
                AssetName::Cidr(c) => c.to_string(),
            }
        )
    }
}
impl PartialEq for AssetName {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AssetName::Domain(a), AssetName::Domain(b)) => a == b,
            (AssetName::Subdomain(a), AssetName::Subdomain(b)) => a == b,
            (AssetName::Url(a), AssetName::Url(b)) => a == b,
            (AssetName::Cidr(a), AssetName::Cidr(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for AssetName {}

impl FromStr for AssetName {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(cidr) = s.parse::<IpNet>() {
            Ok(AssetName::Cidr(cidr))
        } else if let Ok(cidr) = format!("{s}/32").parse::<IpNet>() {
            Ok(AssetName::Cidr(cidr))
        } else if let Ok(url) = url::Url::parse(s) {
            Ok(AssetName::Url(Request {
                url,
                title: None,
                sc: None,
                resp: None,
            }))
        } else if let Ok(domain) = addr::parse_domain_name(s) {
            if let Some(root) = domain.root() {
                if domain.prefix().is_some() {
                    Ok(AssetName::Subdomain(url::Host::parse(s)?))
                } else {
                    Ok(AssetName::Domain(root.to_string()))
                }
            } else {
                Err("Domain doesn't have root".into())
            }
        } else {
            Err("Asset isn't valid".into())
        }
    }
}

impl Ord for AssetName {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self, &other) {
            (AssetName::Domain(s), AssetName::Domain(o)) => s.cmp(o),
            (AssetName::Domain(_), AssetName::Subdomain(_)) => Ordering::Less,
            (AssetName::Domain(_), AssetName::Url(_)) => Ordering::Less,
            (AssetName::Domain(_), AssetName::Cidr(_)) => Ordering::Greater,
            (AssetName::Subdomain(_), AssetName::Domain(_)) => Ordering::Greater,
            (AssetName::Subdomain(s), AssetName::Subdomain(o)) => s.cmp(o),
            (AssetName::Subdomain(_), AssetName::Url(_)) => Ordering::Less,
            (AssetName::Subdomain(_), AssetName::Cidr(_)) => Ordering::Greater,
            (AssetName::Url(_), AssetName::Domain(_)) => Ordering::Greater,
            (AssetName::Url(_), AssetName::Subdomain(_)) => Ordering::Greater,
            (AssetName::Url(s), AssetName::Url(o)) => s.cmp(o),
            (AssetName::Url(_), AssetName::Cidr(_)) => Ordering::Greater,
            (AssetName::Cidr(_), AssetName::Domain(_)) => Ordering::Less,
            (AssetName::Cidr(_), AssetName::Subdomain(_)) => Ordering::Less,
            (AssetName::Cidr(_), AssetName::Url(_)) => Ordering::Less,
            (AssetName::Cidr(s), AssetName::Cidr(o)) => s.cmp(o),
        }
    }
}

impl PartialOrd for AssetName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Asset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Asset {}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

mod test {

    #[test]
    fn merge_test() {
        use super::*;
        let mut s = Asset::from_str("http://b.com/a/b?a=1&b=2&a=3&b=2").unwrap();
        let o = Asset::from_str("http://b.com/a/b?a=3&b=4&a=5&b=4").unwrap();
        s.merge(o);
        assert_eq!(s.name.to_string(), "http://b.com/a/b?a=5&b=4");
    }

    #[test]
    fn asset_ord_0() {
        use super::*;

        assert!(Asset::from_str("http://b.com/a/b").unwrap() > Asset::from_str("b.com").unwrap());

        let mut assets = vec![
            Asset::from_str("http://b.com/a/b").unwrap(),
            Asset::from_str("http://c.com/a/b/c").unwrap(),
            Asset::from_str("http://b.com/a/b").unwrap(),
            Asset::from_str("http://a.com/a").unwrap(),
            Asset::from_str("http://b.com/a/b").unwrap(),
            Asset::from_str("http://a.com/a").unwrap(),
        ];
        assets.sort();
        assert_eq!(
            assets,
            vec![
                Asset::from_str("http://a.com/a").unwrap(),
                Asset::from_str("http://a.com/a").unwrap(),
                Asset::from_str("http://b.com/a/b").unwrap(),
                Asset::from_str("http://b.com/a/b").unwrap(),
                Asset::from_str("http://b.com/a/b").unwrap(),
                Asset::from_str("http://c.com/a/b/c").unwrap(),
            ]
        );

        let mut assets = vec![
            Asset::from_str("http://c.com").unwrap(),
            Asset::from_str("http://a.com").unwrap(),
            Asset::from_str("http://b.com").unwrap(),
        ];
        assets.sort();
        assert_eq!(
            assets,
            vec![
                Asset::from_str("http://a.com").unwrap(),
                Asset::from_str("http://b.com").unwrap(),
                Asset::from_str("http://c.com").unwrap(),
            ]
        );

        let mut assets = vec![
            Asset::from_str("http://a.com/x/y/z").unwrap(),
            Asset::from_str("http://a.com/a/x/y").unwrap(),
            Asset::from_str("http://a.com/x/b").unwrap(),
        ];
        assets.sort();
        assert_eq!(
            assets,
            vec![
                Asset::from_str("http://a.com/x/b").unwrap(),
                Asset::from_str("http://a.com/a/x/y").unwrap(),
                Asset::from_str("http://a.com/x/y/z").unwrap(),
            ]
        );

        let mut assets = vec![
            Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa&r=rose").unwrap(),
            Asset::from_str("http://a.com/x/y/z?l=lisa&j=jisoo&jen=jennie").unwrap(),
            Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa").unwrap(),
        ];
        assets.sort();
        assert_eq!(
            assets,
            vec![
                Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa").unwrap(),
                Asset::from_str("http://a.com/x/y/z?l=lisa&j=jisoo&jen=jennie").unwrap(),
                Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa&r=rose").unwrap(),
            ]
        );
        assert_eq!(
            assets,
            vec![
                Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa").unwrap(),
                Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa&r=rose").unwrap(),
                Asset::from_str("http://a.com/x/y/z?l=lisa&j=jisoo&jen=jennie").unwrap(),
            ]
        );
    }

    #[test]
    fn asset_eq() {
        use super::*;

        assert_ne!(
            Asset::from_str("http://x.com/feature/printableindex.ch2").unwrap(),
            Asset::from_str("http://x.com/contactus/index.ch2").unwrap(),
        );

        assert_ne!(
            Asset::from_str("http://x.com/upload/locationpageimage1/21-skyline2.jpg").unwrap(),
            Asset::from_str("http://x.com/feature/printableindex.ch2").unwrap(),
        );

        assert_ne!(
            Asset::from_str("http://x.com/a/b/c/e/f").unwrap(),
            Asset::from_str("http://x.com/a/b/d/c").unwrap(),
        );

        assert_ne!(
            Asset::from_str("http://x.com/a/b").unwrap(),
            Asset::from_str("http://x.com/c/d").unwrap(),
        );

        assert_ne!(
            Asset::from_str("http://x.com/a/b").unwrap(),
            Asset::from_str("http://x.com/c/d").unwrap(),
        );
    }

    #[test]
    fn asset_name() {
        use super::*;
        use crate::asset::AssetName;
        use std::str::FromStr;

        assert_eq!(
            AssetName::from_str("192.168.1.0/32").unwrap(),
            AssetName::Cidr("192.168.1.0/32".parse::<IpNet>().unwrap())
        );

        assert_eq!(
            AssetName::from_str("google.com").unwrap(),
            AssetName::Domain("google.com".to_string())
        );
        assert_eq!(
            AssetName::from_str("sub.google.com").unwrap(),
            AssetName::Subdomain(url::Host::parse("sub.google.com").unwrap())
        );

        assert_eq!(
            AssetName::from_str("https://sub.google.com").unwrap(),
            AssetName::Url(Request {
                url: url::Url::parse("https://sub.google.com").unwrap(),
                resp: None,
                sc: None,
                title: None
            })
        );
    }
}
