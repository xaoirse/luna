use super::*;

#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
pub struct Asset {
    pub name: AssetName,
    #[clap(long, short)]
    pub tags: Vec<Tag>,

    #[clap(skip)]
    pub update: Time,
    #[clap(skip)]
    pub start: Time,
}

impl FromStr for Asset {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            name: AssetName::from_str(s)?,
            tags: vec![],
            update: Time::default(),
            start: Time::default(),
        })
    }
}

impl Asset {
    pub fn merge(&mut self, other: Self) {
        self.update = self.update.max(other.update);
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
        self.tags.par_iter_mut().find_any(|t| t.name == name)
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),

            1 => format!(
                "{}
    Tags:  [{}{}
    Update: {}
    Start:  {}
    ",
                self.name,
                self.tags
                    .iter()
                    .map(|s| format!("\n        {}", s.stringify(1)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.tags.is_empty() { "]" } else { "\n    ]" },
                self.update.0.to_rfc2822(),
                self.start.0.to_rfc2822(),
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
    #[serde(with = "super::serde_cidr")]
    Cidr(IpCidr),
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
        if let Ok(cidr) = s.parse::<IpCidr>() {
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

mod test {

    #[test]
    fn asset_name() {
        use super::*;
        use crate::asset::AssetName;
        use std::str::FromStr;

        let cidr = "192.168.1.0/32".parse::<IpCidr>().unwrap();
        dbg!(cidr);

        assert_eq!(
            AssetName::from_str("google.com").unwrap(),
            AssetName::Domain("google.com".to_string())
        );
        assert_eq!(
            AssetName::from_str("sub.google.com").unwrap(),
            AssetName::Subdomain(url::Host::parse("sub.google.com").unwrap())
        );
    }
}
