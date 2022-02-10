use clap::arg_enum;
use structopt::StructOpt;

use super::*;

#[derive(Debug, StructOpt, Default)]
pub struct Filter {
    #[structopt(possible_values = &Fields::variants(), case_insensitive = true, help="Case Insensitive")]
    pub field: Fields,
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(long, short)]
    pub program: Option<String>,
    #[structopt(long)]
    pub program_platform: Option<String>,
    #[structopt(long)]
    pub program_handle: Option<String>,
    #[structopt(long)]
    pub program_type: Option<String>,
    #[structopt(long)]
    pub program_url: Option<String>,
    #[structopt(long)]
    pub program_icon: Option<String>,
    #[structopt(long)]
    pub program_bounty: Option<String>,
    #[structopt(long)]
    pub program_state: Option<String>,

    #[structopt(long, short)]
    pub scope: Option<String>,
    #[structopt(long)]
    pub scope_type: Option<String>,
    #[structopt(long)]
    pub scope_bounty: Option<String>,
    #[structopt(long)]
    pub scope_severity: Option<String>,

    #[structopt(long)]
    pub sub: Option<String>,
    #[structopt(long)]
    pub sub_typ: Option<String>,

    #[structopt(long)]
    pub ip: Option<String>,

    #[structopt(long)]
    pub port: Option<String>,
    #[structopt(long)]
    pub service_name: Option<String>,
    #[structopt(long)]
    pub service_protocol: Option<String>,
    #[structopt(long)]
    pub service_banner: Option<String>,

    #[structopt(long)]
    pub url: Option<String>,
    #[structopt(long)]
    pub title: Option<String>,
    #[structopt(long, short = "c")]
    pub status_code: Option<String>,
    #[structopt(long, short = "r")]
    pub response: Option<String>,

    #[structopt(long)]
    pub tech: Option<String>,
    #[structopt(long)]
    pub tech_version: Option<String>,

    #[structopt(long, short)]
    pub updated_at: Option<i64>,

    #[structopt(long, short)]
    pub started_at: Option<i64>,
}

impl Filter {
    pub fn scope_is_none(&self) -> bool {
        self.scope.is_none()
            && self.scope_type.is_none()
            && self.scope_bounty.is_none()
            && self.sub_is_none()
    }
    pub fn sub_is_none(&self) -> bool {
        self.sub.is_none() && self.sub_typ.is_none() && self.host_is_none() && self.url_is_none()
    }

    pub fn url_is_none(&self) -> bool {
        self.url.is_none()
            && self.title.is_none()
            && self.status_code.is_none()
            && self.response.is_none()
            && self.tech_is_none()
    }

    pub fn tech_is_none(&self) -> bool {
        self.tech.is_none() && self.tech_version.is_none()
    }

    pub fn host_is_none(&self) -> bool {
        self.ip.is_none() && self.port.is_none() && self.service_is_none()
    }

    pub fn service_is_none(&self) -> bool {
        self.service_name.is_none()
            && self.service_protocol.is_none()
            && self.service_banner.is_none()
    }
}

arg_enum! {
    #[derive(Debug, Clone,Copy)]
    pub enum Fields {
        None,
        Keyword,
        Tech,
        Service,
        IP,
        Url,
        Sub,
        Scope,
        Program,
    }
}

impl Default for Fields {
    fn default() -> Self {
        Self::Scope
    }
}

impl From<&Fields> for &str {
    fn from(f: &Fields) -> Self {
        match f {
            Fields::Program => "program",
            Fields::Scope => "scope",
            Fields::Sub => "sub",
            Fields::Url => "url",
            Fields::IP => "ip",
            Fields::Keyword => "keyword",
            Fields::Service => "port",
            Fields::None => "",
            Fields::Tech => todo!(),
        }
    }
}

impl Fields {
    pub fn substitution(&self) -> String {
        let f: &str = self.into();
        format!("${{{}}}", f)
    }
}

#[derive(Default)]
pub struct FilterRegex {
    pub field: Fields,
    pub verbose: u8,

    pub program: Option<regex::Regex>,
    pub program_platform: Option<regex::Regex>,
    pub program_handle: Option<regex::Regex>,
    pub program_type: Option<regex::Regex>,
    pub program_url: Option<regex::Regex>,
    pub program_icon: Option<regex::Regex>,
    pub program_bounty: Option<regex::Regex>,
    pub program_state: Option<regex::Regex>,

    pub scope: Option<regex::Regex>,
    pub scope_type: Option<regex::Regex>,
    pub scope_bounty: Option<regex::Regex>,
    pub scope_severity: Option<regex::Regex>,

    pub sub: Option<regex::Regex>,
    pub sub_typ: Option<regex::Regex>,

    pub ip: Option<regex::Regex>,

    pub port: Option<regex::Regex>,
    pub service_name: Option<regex::Regex>,
    pub service_protocol: Option<regex::Regex>,
    pub service_banner: Option<regex::Regex>,

    pub url: Option<regex::Regex>,
    pub title: Option<regex::Regex>,
    pub status_code: Option<regex::Regex>,
    pub response: Option<regex::Regex>,

    pub tech: Option<regex::Regex>,
    pub tech_version: Option<regex::Regex>,

    pub updated_at: Option<i64>,
    pub started_at: Option<i64>,
}
impl FilterRegex {
    pub fn scope_is_none(&self) -> bool {
        self.scope.is_none()
            && self.scope_type.is_none()
            && self.scope_bounty.is_none()
            && self.sub_is_none()
    }
    pub fn sub_is_none(&self) -> bool {
        self.sub.is_none() && self.sub_typ.is_none() && self.host_is_none() && self.url_is_none()
    }

    pub fn url_is_none(&self) -> bool {
        self.url.is_none()
            && self.title.is_none()
            && self.status_code.is_none()
            && self.response.is_none()
            && self.tech_is_none()
    }

    pub fn tech_is_none(&self) -> bool {
        self.tech.is_none() && self.tech_version.is_none()
    }

    pub fn host_is_none(&self) -> bool {
        self.ip.is_none() && self.port.is_none() && self.service_is_none()
    }

    pub fn service_is_none(&self) -> bool {
        self.service_name.is_none()
            && self.service_protocol.is_none()
            && self.service_banner.is_none()
    }
}

use regex::Regex;
impl TryFrom<Filter> for FilterRegex {
    type Error = Errors;

    fn try_from(f: Filter) -> Result<Self, Self::Error> {
        let program = match f.program {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let program_platform = match f.program_platform {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let program_handle = match f.program_handle {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let program_type = match f.program_type {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let program_url = match f.program_url {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let program_icon = match f.program_icon {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let program_bounty = match f.program_bounty {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let program_state = match f.program_state {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };

        let scope = match f.scope {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let scope_type = match f.scope_type {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let scope_bounty = match f.scope_bounty {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let scope_severity = match f.scope_severity {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };

        let sub = match f.sub {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let sub_typ = match f.sub_typ {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };

        let ip = match f.ip {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };

        let port = match f.port {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let service_name = match f.service_name {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let service_protocol = match f.service_protocol {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let service_banner = match f.service_banner {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };

        let url = match f.url {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let title = match f.title {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let status_code = match f.status_code {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let response = match f.response {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };

        let tech = match f.tech {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };
        let tech_version = match f.tech_version {
            Some(ref p) => Some(Regex::new(p)?),
            None => None,
        };

        Ok(Self {
            field: f.field,
            verbose: f.verbose,

            program,
            program_platform,
            program_handle,
            program_type,
            program_url,
            program_icon,
            program_bounty,
            program_state,

            scope,
            scope_type,
            scope_bounty,
            scope_severity,

            sub,
            sub_typ,

            ip,

            port,
            service_name,
            service_protocol,
            service_banner,

            url,
            title,
            status_code,
            response,

            tech,
            tech_version,

            updated_at: f.updated_at,
            started_at: f.started_at,
        })
    }
}
