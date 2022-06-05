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
                    .map(|s| format!("\n        {}", s.stringify(1)))
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
                    .map(|s| format!("\n        {}", s.stringify(2)))
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
                    .map(|s| format!("\n        {}", s.stringify(2)))
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

use std::cmp::Ordering;

impl Ord for Asset {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.name, &other.name) {
            (AssetName::Domain(s), AssetName::Domain(o)) => s.cmp(o),
            (AssetName::Domain(_), AssetName::Subdomain(_)) => Ordering::Greater,
            (AssetName::Domain(_), AssetName::Url(_)) => Ordering::Greater,
            (AssetName::Domain(_), AssetName::Cidr(_)) => Ordering::Less,
            (AssetName::Subdomain(_), AssetName::Domain(_)) => Ordering::Less,
            (AssetName::Subdomain(s), AssetName::Subdomain(o)) => s.cmp(o),
            (AssetName::Subdomain(_), AssetName::Url(_)) => Ordering::Greater,
            (AssetName::Subdomain(_), AssetName::Cidr(_)) => Ordering::Less,
            (AssetName::Url(_), AssetName::Domain(_)) => Ordering::Less,
            (AssetName::Url(_), AssetName::Subdomain(_)) => Ordering::Less,
            (AssetName::Url(s), AssetName::Url(o)) => {
                if self == other {
                    return Ordering::Equal;
                }

                if s.url[..url::Position::BeforePath] != o.url[..url::Position::BeforePath] {
                    return s.url[..url::Position::BeforePath]
                        .cmp(&o.url[..url::Position::BeforePath]);
                }

                match (s.url.path_segments(), o.url.path_segments()) {
                    (None, None) => (),
                    (None, Some(_)) => return Ordering::Less,
                    (Some(_), None) => return Ordering::Greater,
                    (Some(s_path), Some(o_path)) => {
                        let s_path: Vec<_> = s_path.collect();
                        let o_path: Vec<_> = o_path.collect();

                        if s_path.len() > o_path.len() {
                            return Ordering::Greater;
                        } else if s_path.len() < o_path.len() {
                            return Ordering::Less;
                        } else if s_path.len() > 1
                            && s_path.split_last().unwrap().1 != o_path.split_last().unwrap().1
                        {
                            return s_path
                                .split_last()
                                .unwrap()
                                .1
                                .cmp(o_path.split_last().unwrap().1);
                        }
                    }
                }

                let mut s_params: Vec<_> = s
                    .url
                    .query_pairs()
                    .map(|(k, _)| k)
                    .filter(|s| !s.is_empty())
                    .collect();
                let mut o_params: Vec<_> = o
                    .url
                    .query_pairs()
                    .map(|(k, _)| k)
                    .filter(|s| !s.is_empty())
                    .collect();

                if s_params.len() != o_params.len() {
                    return s_params.len().cmp(&o_params.len());
                }

                if s_params.is_empty() {
                    return Ordering::Equal;
                }

                s_params.sort();
                o_params.sort();
                let s = s_params.into_iter().collect::<String>();
                let o = o_params.into_iter().collect::<String>();

                s.cmp(&o)
            }
            (AssetName::Url(_), AssetName::Cidr(_)) => Ordering::Less,
            (AssetName::Cidr(_), AssetName::Domain(_)) => Ordering::Greater,
            (AssetName::Cidr(_), AssetName::Subdomain(_)) => Ordering::Greater,
            (AssetName::Cidr(_), AssetName::Url(_)) => Ordering::Greater,
            (AssetName::Cidr(s), AssetName::Cidr(o)) => s.cmp(o),
        }
    }
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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
    fn asset_ord_0() {
        use super::*;

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
        assert_ne!(
            assets,
            vec![
                Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa").unwrap(),
                Asset::from_str("http://a.com/x/y/z?j=jisoo&l=lisa&r=rose").unwrap(),
                Asset::from_str("http://a.com/x/y/z?l=lisa&j=jisoo&jen=jennie").unwrap(),
            ]
        );
    }

    #[test]
    fn asset_ord_1() {
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
    fn asset_ord_2() {
        use super::*;

        let assets = vec![
            // Asset::from_str("http://44charlesleasing.manulifecentre.com/feature/printableindex.ch2").unwrap(),
            // Asset::from_str("http://44charlesleasing.manulifecentre.com/leasing/leasingbrochure.ch2").unwrap(),
            Asset::from_str("http://44charlesleasing.manulifecentre.com/leasing/leasingbrochurepopup.ch2?selectedCounter=0").unwrap(),
            // Asset::from_str("http://44charlesleasing.manulifecentre.com/contactus/index.ch2").unwrap(),
            Asset::from_str("http://44charlesleasing.manulifecentre.com/").unwrap(),
            Asset::from_str("http://44charlesleasing.manulifecentre.com/").unwrap(),
            Asset::from_str("http://44charlesleasing.manulifecentre.com/contactus/index.ch2").unwrap(),
            Asset::from_str("http://44charlesleasing.manulifecentre.com/leasing/leasingbrochurepopup.ch2?selectedCounter=0").unwrap(),
            Asset::from_str("http://44charlesleasing.manulifecentre.com/upload/buildinglogoimage/161-44logonew.gif").unwrap(),
            Asset::from_str("http://44charlesleasing.manulifecentre.com/designfiles/design001/image/email.gif").unwrap(),
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

        assert_eq!(
            p2.assets,
            vec![
                Asset::from_str("http://44charlesleasing.manulifecentre.com/").unwrap(),
                Asset::from_str("http://44charlesleasing.manulifecentre.com/contactus/index.ch2").unwrap(),
                Asset::from_str("http://44charlesleasing.manulifecentre.com/leasing/leasingbrochurepopup.ch2?selectedCounter=0").unwrap(),
                Asset::from_str("http://44charlesleasing.manulifecentre.com/upload/buildinglogoimage/161-44logonew.gif").unwrap(),
                Asset::from_str("http://44charlesleasing.manulifecentre.com/designfiles/design001/image/email.gif").unwrap(),
            ],
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
