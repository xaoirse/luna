use super::*;
use chrono::Utc;
use clap::Parser;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize, Serialize)]
pub struct Luna {
    pub name: String,
    pub version: String,
    pub status: String,
    pub programs: Vec<Program>,

    pub update: Time,
    pub start: Time,
}
impl Default for Luna {
    fn default() -> Self {
        Self {
            name: "Luna".to_string(),
            version: VERSION.to_string(),
            status: "In the service of Selemene".to_string(),
            programs: vec![],
            update: Time::default(),
            start: Time::default(),
        }
    }
}

impl Luna {
    pub fn insert_program(&mut self, program: Program) -> Result<(), Errors> {
        if let Some(p) = self.program_by_name(&program.name) {
            p.merge(program);
        } else {
            self.programs.push(program);
        }
        Ok(())
    }

    pub fn insert_asset(&mut self, asset: Asset, program: Option<Program>) -> Result<(), Errors> {
        if let Some(a) = self.asset_by_name(&asset.name) {
            a.merge(asset);
        } else {
            match &asset.name {
                AssetName::Url(request) => {
                    if let Some(host) = request.url.host_str() {
                        self.insert_asset(Asset::from_str(host)?, None)?;
                        if let Some(domain) = asset.name.domain() {
                            if let Some(pr) = self.program_by_asset(&domain) {
                                pr.assets.push(asset);
                                return Ok(());
                            }
                        }
                    }
                    return Err("oos".into());
                }

                AssetName::Subdomain(_) => {
                    if let Some(domain) = asset.name.domain() {
                        if let Some(pr) = self.program_by_asset(&domain) {
                            pr.assets.push(asset);
                            return Ok(());
                        }
                    }

                    return Err("oos".into());
                }

                AssetName::Domain(_) => {
                    if let Some(mut pr) = program {
                        if let Some(pr) = self.program_by_name(&pr.name) {
                            pr.assets.push(asset);
                        } else {
                            pr.assets.push(asset);
                            self.insert_program(pr)?;
                        }
                        return Ok(());
                    }
                    return Err("oop".into());
                }
                AssetName::Cidr(_) => {
                    if let Some(mut pr) = program {
                        if let Some(pr) = self.program_by_name(&pr.name) {
                            pr.assets.push(asset);
                        } else {
                            pr.assets.push(asset);
                            self.insert_program(pr)?;
                        }
                        return Ok(());
                    }
                    return Err("oop".into());
                }
            }
        }
        Ok(())
    }

    pub fn insert_tag(&mut self, tag: Tag, asset: &AssetName) -> Result<(), Errors> {
        if let Some(asset) = self.asset_by_name(asset) {
            asset.tags.push(tag);
        } else if let Some(domain) = asset.domain() {
            if let Some(pr) = self.program_by_asset(&domain) {
                let asset = Asset {
                    name: asset.to_owned(),
                    tags: vec![tag],
                    update: Time::default(),
                    start: Time::default(),
                };
                pr.assets.push(asset);
                return Ok(());
            }
        }

        Err("oop".into())
    }

    pub fn program_by_name(&mut self, name: &str) -> Option<&mut Program> {
        self.programs.par_iter_mut().find_any(|p| p.name == name)
    }
    pub fn program_by_asset(&mut self, asset: &AssetName) -> Option<&mut Program> {
        self.programs.par_iter_mut().find_any(|p| {
            p.assets
                .par_iter()
                .any(|a| a.name.domain() == asset.domain())
        })
    }
    pub fn asset_by_name(&mut self, name: &AssetName) -> Option<&mut Asset> {
        self.programs
            .par_iter_mut()
            .flat_map(|p| &mut p.assets)
            .find_any(|a| &a.name == name)
    }

    pub fn programs(&self, filter: &Filter) -> Vec<&Program> {
        self.programs
            .iter()
            .filter(|p| filter.program(p))
            .filter(|a| date(&a.update, &filter.update) || date(&a.start, &filter.start))
            .take(filter.n)
            .collect()
    }
    pub fn assets(&self, field: Field, filter: &Filter) -> Vec<&Asset> {
        self.programs
            .iter()
            .filter(|p| filter.program(p))
            .flat_map(|p| &p.assets)
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
    pub fn tags(&self, filter: &Filter) -> Vec<&Tag> {
        self.programs
            .iter()
            .filter(|p| filter.program(p))
            .flat_map(|p| &p.assets)
            .filter(|a| filter.asset(a))
            .flat_map(|a| &a.tags)
            .filter(|t| filter.tag(t))
            .filter(|a| date(&a.update, &filter.update) || date(&a.start, &filter.start))
            .take(filter.n)
            .collect()
    }
    pub fn find(&self, field: Field, filter: &Filter, v: u8) -> Vec<String> {
        match field {
            Field::Program => self
                .programs(filter)
                .iter()
                .map(|f| f.stringify(v))
                .collect(),
            Field::None => vec!["".to_string()],
            Field::Tag => self.tags(filter).iter().map(|f| f.stringify(v)).collect(),
            Field::Value => self
                .programs(filter)
                .iter()
                .map(|f| f.stringify(v))
                .collect(),
            _ => self
                .assets(field, filter)
                .iter()
                .map(|f| f.stringify(v))
                .collect(),
        }
    }
    pub fn save_as(&self, path: &str) -> Result<usize, Errors> {
        let str = serde_json::to_string(&self)?;

        if !Opt::parse().no_backup && std::path::Path::new(path).exists() {
            let copy_path = match path.rsplit_once('.') {
                Some((a, b)) => format!("{}_{}.{}", a, chrono::Local::now().to_rfc2822(), b),
                None => format!("{}_{}", path, Utc::now().to_rfc2822()),
            };
            std::fs::copy(path, copy_path)?;
        }
        match std::fs::File::options()
            .write(true)
            .truncate(true)
            .open(path)
        {
            Ok(mut file) => Ok(file.write(str.as_bytes())?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Ok(std::fs::File::create(path)?.write(str.as_bytes())?)
            }
            Err(err) => Err(Box::new(err)),
        }
    }

    pub fn save(&self) {
        let opt = Opt::parse();
        let output = opt.output.as_ref().unwrap_or(&opt.input);

        if let Err(err) = self.save_as(output) {
            error!("Error while saving: {}", err);
        } else {
            info!("Saved in \"{}\" successfully.", output);
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Errors> {
        let file = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file)?)
    }

    pub fn parse() -> Luna {
        let opt = Opt::parse();

        match Luna::from_file(&opt.input) {
            Ok(luna) => {
                info!("Luna loaded successfully.");
                luna
            }
            Err(err) => {
                if err.to_string() == "No such file or directory (os error 2)" {
                    warn!("Can't load Luna from file! New filw will be generated.")
                } else {
                    error!("Can't load Luna from file!: {}", err);
                }
                Luna::default()
            }
        }
    }

    pub fn remove() {
        todo!()
    }

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!("{} {}", self.name, self.version),
            2 => format!(
                "{}  {}
    Status:   {}
    Programs: {}
    Domains:  {}
    CIDRs:    {}
    Subs:     {}
    URLs:     {}
    Tags:     {}
    Update:   {}
    Start:    {}
    ",
                self.name,
                self.version,
                self.status,
                self.programs.len(),
                self.find(Field::Domain, &Filter::default(), 0).len(),
                self.find(Field::Cidr, &Filter::default(), 0).len(),
                self.find(Field::Sub, &Filter::default(), 0).len(),
                self.find(Field::Url, &Filter::default(), 0).len(),
                self.find(Field::Tag, &Filter::default(), 0).len(),
                self.update.0.to_rfc2822(),
                self.start.0.to_rfc2822(),
            ),
            3 => format!(
                "{}  {}
    Status:   {}
    Programs: [{}{}
    Domains:  {}
    CIDRs:    {}
    Subs:     {}
    URLs:     {}
    Tags:     {}
    Update:   {}
    Start:    {}
    ",
                self.name,
                self.version,
                self.status,
                self.programs
                    .iter()
                    .filter(|p| !p.name.is_empty())
                    .map(|p| format!("\n        {}", p.stringify(1)))
                    .collect::<Vec<String>>()
                    .join(""),
                if self.programs.iter().filter(|p| !p.name.is_empty()).count() == 0 {
                    "]"
                } else {
                    "\n    ]"
                },
                self.find(Field::Domain, &Filter::default(), 0).len(),
                self.find(Field::Cidr, &Filter::default(), 0).len(),
                self.find(Field::Sub, &Filter::default(), 0).len(),
                self.find(Field::Url, &Filter::default(), 0).len(),
                self.find(Field::Tag, &Filter::default(), 0).len(),
                self.update.0.to_rfc2822(),
                self.start.0.to_rfc2822(),
            ),
            _ => format!("{:#?}", self),
        }
    }
}
