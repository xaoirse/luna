use super::*;

#[derive(Clone, ArgEnum, Copy)]
pub enum Field {
    Luna,
    Program,
    Asset,
    Domain,
    Cidr,
    Sub,
    Url,
    Tag,
    Value,
    None,
}

impl Field {
    pub fn substitution(&self) -> &'static str {
        match self {
            Field::Luna => "${luna}",
            Field::Program => "${program}",
            Field::Asset => "${asset}",
            Field::Domain => "${domain}",
            Field::Cidr => "${cidr}",
            Field::Sub => "${sub}",
            Field::Url => "${url}",
            Field::Tag => "${tag}",
            Field::Value => "${value}",
            Field::None => "${none}",
        }
    }
}

#[derive(Parser)]
pub struct Filter {
    #[clap(short, default_value = "18446744073709551615")]
    pub n: usize,

    #[clap(long, short)]
    pub program: Option<Regex>,
    #[clap(long)]
    pub platform: Option<Regex>,
    #[clap(long = "type")]
    pub typ: Option<Regex>,
    #[clap(long)]
    pub url: Option<Regex>,
    #[clap(long)]
    pub handle: Option<Regex>,
    #[clap(long)]
    pub bounty: Option<Regex>,
    #[clap(long)]
    pub state: Option<Regex>,
    #[clap(long, short)]
    pub asset: Option<Regex>,
    #[clap(long, name = "STATUS CODE")]
    pub sc: Option<Regex>,
    #[clap(long)]
    pub title: Option<Regex>,
    #[clap(long, name = "RESPONSE")]
    pub resp: Option<Regex>,
    #[clap(long)]
    pub tag: Option<Regex>,
    #[clap(long = "sv")]
    pub severity: Option<Regex>,
    #[clap(long)]
    pub value: Option<Regex>,

    #[clap(long, short, name = "HOURS", help = "How many hours ago?")]
    pub start: Option<Time>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            n: 18446744073709551615,
            program: None,
            platform: None,
            typ: None,
            url: None,
            handle: None,
            bounty: None,
            state: None,
            asset: None,
            sc: None,
            title: None,
            resp: None,
            tag: None,
            severity: None,
            value: None,

            start: Some(Time(Utc::now() - chrono::Duration::weeks(5400))),
        }
    }
}

pub enum Regex {
    Cidr(IpNet),
    Regex(regex::Regex),
    Empty,
}
impl Default for Regex {
    fn default() -> Regex {
        Regex::Empty
    }
}

impl FromStr for Regex {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::Empty)
        } else if s.contains('.') {
            if let Ok(cidr) = s.parse::<IpNet>() {
                Ok(Self::Cidr(cidr))
            } else if let Ok(cidr) = format!("{}/32", s).parse::<IpNet>() {
                Ok(Self::Cidr(cidr))
            } else {
                Ok(Self::Regex(regex::Regex::new(&format!("(?i){}", s))?))
            }
        } else {
            Ok(Self::Regex(regex::Regex::new(&format!("(?i){}", s))?))
        }
    }
}

trait RegexOpt {
    fn is_empty(&self) -> bool;
    fn cidr_match(&self, cidr: &IpNet) -> bool;
    fn string_match(&self, str: &str) -> bool;
    fn option_match(&self, str: &Option<String>) -> bool;
}
impl RegexOpt for Option<Regex> {
    fn is_empty(&self) -> bool {
        if let Some(re) = self {
            matches!(re, Regex::Empty)
        } else {
            true
        }
    }

    fn cidr_match(&self, cidr: &IpNet) -> bool {
        if let Some(re) = self {
            match re {
                Regex::Cidr(fcidr) => fcidr.contains(cidr) || cidr.contains(fcidr),
                Regex::Regex(_) => false,
                Regex::Empty => true,
            }
        } else {
            true
        }
    }

    fn string_match(&self, str: &str) -> bool {
        if let Some(re) = self {
            match re {
                Regex::Cidr(_) => false,
                Regex::Regex(re) => re.is_match(str),
                Regex::Empty => true,
            }
        } else {
            true
        }
    }

    fn option_match(&self, str: &Option<String>) -> bool {
        if let Some(re) = self {
            match (re, str) {
                (Regex::Cidr(_), _) => false,
                (Regex::Regex(re), Some(str)) => re.is_match(str),
                (Regex::Regex(_), None) => false,
                _ => true,
            }
        } else {
            true
        }
    }
}

impl Filter {
    pub fn program(&self, program: &Program) -> bool {
        self.program.string_match(&program.name)
            && self.platform.option_match(&program.platform)
            && self.typ.option_match(&program.typ)
            && self.url.option_match(&program.url)
            && self.handle.option_match(&program.handle)
            && self.bounty.option_match(&program.bounty)
            && self.state.option_match(&program.state)
            && (self.asset_is_empty() || program.assets.iter().any(|a| self.asset(a)))
    }
    pub fn asset(&self, asset: &Asset) -> bool {
        (match &asset.name {
            AssetName::Domain(d) => self.asset.string_match(d),
            AssetName::Subdomain(h) => self.asset.string_match(&h.to_string()),
            AssetName::Url(req) => {
                self.asset.string_match(req.url.as_str())
                    && self.sc.option_match(&req.sc)
                    && self.title.option_match(&req.title)
                    && self.resp.option_match(&req.resp)
            }
            AssetName::Cidr(c) => self.asset.cidr_match(c),
        }) && (self.tag_is_empty() || asset.tags.iter().any(|a| self.tag(a)))
    }
    pub fn tag(&self, tag: &Tag) -> bool {
        self.tag.string_match(&tag.name)
            && self.severity.option_match(&tag.severity)
            && (self.value.is_empty() || tag.values.iter().any(|v| self.value.string_match(v)))
    }
    pub fn value(&self, str: &str) -> bool {
        self.value.string_match(str)
    }

    pub fn asset_is_empty(&self) -> bool {
        self.asset.is_empty()
            && self.url.is_empty()
            && self.sc.is_empty()
            && self.title.is_empty()
            && self.resp.is_empty()
            && self.tag_is_empty()
    }
    pub fn tag_is_empty(&self) -> bool {
        self.tag.is_empty() && self.severity.is_empty() && self.value.is_empty()
    }
}

mod test {

    #[test]
    fn asset() {
        use super::*;

        let mut asset = Asset::from_str("google.com").unwrap();

        asset.tags.push(Tag {
            name: "sql".to_string(),
            severity: Some("High".to_string()),
            ..Default::default()
        });

        let filter1 = Filter {
            asset: Some(Regex::from_str("goo").unwrap()),
            ..Default::default()
        };
        assert!(filter1.asset(&asset));

        let filter2 = Filter {
            severity: Some(Regex::from_str("Hi").unwrap()),
            ..Default::default()
        };
        assert!(filter2.asset(&asset));

        let filter3 = Filter {
            severity: Some(Regex::from_str("L").unwrap()),
            ..Default::default()
        };
        assert!(!filter3.asset(&asset));
    }

    #[test]
    fn asset_is_empty() {
        use super::*;
        let f = Filter {
            asset: Some(Regex::from_str("mia").unwrap()),
            ..Default::default()
        };
        assert!(!f.asset_is_empty());
        assert!(f.tag_is_empty());

        let f = Filter {
            asset: Some(Regex::from_str("mia").unwrap()),
            severity: Some(Regex::from_str("mia").unwrap()),
            ..Default::default()
        };
        assert!(!f.asset_is_empty());
        assert!(!f.tag_is_empty());

        let f = Filter {
            severity: Some(Regex::from_str("mia").unwrap()),
            ..Default::default()
        };
        assert!(!f.asset_is_empty());
        assert!(!f.tag_is_empty());
    }

    #[test]
    fn regex_match() {
        use super::*;

        let str = "Mia";
        let regex = Some(Regex::from_str("m").unwrap());
        assert!(regex.string_match(str));

        let str = "123";
        let regex = Some(Regex::from_str("2").unwrap());
        assert!(regex.string_match(str));

        let str = "123";
        let regex = Some(Regex::from_str("5").unwrap());
        assert!(!regex.string_match(str));

        let c = "1.1.1.0/24".parse::<IpNet>().unwrap();
        let regex = Some(Regex::from_str("1.1.1.1/32").unwrap());
        assert!(regex.cidr_match(&c));

        let str = "1.1.5.0/24";
        let regex = Some(Regex::from_str("1.1.1.1/32").unwrap());
        assert!(!regex.string_match(str))
    }
}
