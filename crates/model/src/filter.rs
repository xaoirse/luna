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
    #[clap(long, short, default_value = "")]
    pub program: Regex,
    #[clap(long, default_value = "")]
    pub platform: Regex,
    #[clap(long = "type", default_value = "")]
    pub typ: Regex,
    #[clap(long, default_value = "")]
    pub url: Regex,
    #[clap(long, default_value = "")]
    pub handle: Regex,
    #[clap(long, default_value = "")]
    pub bounty: Regex,
    #[clap(long, default_value = "")]
    pub state: Regex,
    #[clap(long, short, default_value = "")]
    pub asset: Regex,
    #[clap(long, default_value = "")]
    pub sc: Regex,
    #[clap(long, default_value = "")]
    pub title: Regex,
    #[clap(long, default_value = "")]
    pub resp: Regex,
    #[clap(long, default_value = "")]
    pub tag: Regex,
    #[clap(long, default_value = "")]
    pub severity: Regex,
    #[clap(long, default_value = "")]
    pub value: Regex,
    #[clap(long, short)]
    pub update: Option<i64>,
    #[clap(long, short)]
    pub start: Option<i64>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            n: 18446744073709551615,
            program: Regex::None,
            platform: Regex::None,
            typ: Regex::None,
            url: Regex::None,
            handle: Regex::None,
            bounty: Regex::None,
            state: Regex::None,
            asset: Regex::None,
            sc: Regex::None,
            title: Regex::None,
            resp: Regex::None,
            tag: Regex::None,
            severity: Regex::None,
            value: Regex::None,

            update: None,
            start: None,
        }
    }
}

pub enum Regex {
    None,
    Cidr(IpCidr),
    Some(regex::Regex),
}
impl Default for Regex {
    fn default() -> Self {
        Self::None
    }
}

impl FromStr for Regex {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::None)
        } else if let Ok(cidr) = s.parse::<IpCidr>() {
            Ok(Self::Cidr(cidr))
        } else {
            Ok(Self::Some(regex::Regex::new(&format!("(?i){}", s))?))
        }
    }
}
impl Regex {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn string_match(&self, str: &str) -> bool {
        if let Self::Some(regex) = self {
            regex.is_match(str)
        } else {
            matches!(self, Self::None)
        }
    }

    fn option_match(&self, str: &Option<String>) -> bool {
        match (self, str) {
            (Self::Some(re), Some(str)) => re.is_match(str),
            (Self::None, _) => true,
            _ => false,
        }
    }

    fn asset_match(&self, asset: &AssetName) -> bool {
        match (self, asset) {
            (Regex::None, _) => true,
            (Self::Some(re), AssetName::Domain(d)) => re.is_match(d),
            (Regex::Some(re), AssetName::Subdomain(h)) => re.is_match(&h.to_string()),
            (Regex::Some(re), AssetName::Url(req)) => re.is_match(req.url.as_str()),
            (Regex::Some(re), AssetName::Cidr(cidr)) => re.is_match(&cidr.to_string()),
            (Regex::Cidr(a), AssetName::Cidr(b)) => {
                a.contains(&b.first_address()) || b.contains(&a.first_address())
            }
            _ => false,
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
            && (self.asset_is_none() || program.assets.par_iter().any(|a| self.asset(a)))
    }
    pub fn asset(&self, asset: &Asset) -> bool {
        self.asset.asset_match(&asset.name)
            && (self.tag_is_none() || asset.tags.par_iter().any(|t| self.tag(t)))
    }
    pub fn tag(&self, tag: &Tag) -> bool {
        self.tag.string_match(&tag.name)
            && self.severity.option_match(&tag.severity)
            && (self.value.is_none() || tag.values.par_iter().any(|v| self.value.string_match(v)))
    }
    pub fn value(&self, str: &str) -> bool {
        self.value.string_match(str)
    }

    pub fn asset_is_none(&self) -> bool {
        self.asset.is_none()
            && self.url.is_none()
            && self.sc.is_none()
            && self.title.is_none()
            && self.resp.is_none()
            && self.tag_is_none()
    }
    pub fn tag_is_none(&self) -> bool {
        self.tag.is_none() && self.severity.is_none() && self.value.is_none()
    }
}