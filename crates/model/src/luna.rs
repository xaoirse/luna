use super::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize, Serialize)]
pub struct Luna {
    pub name: String,
    pub version: String,
    pub status: String,

    pub programs: Vec<Program>,

    pub start: Time,
}
impl Default for Luna {
    fn default() -> Self {
        Self {
            name: "Luna".to_string(),
            version: VERSION.to_string(),
            status: "In the service of Selemene".to_string(),
            programs: vec![],
            start: Time::default(),
        }
    }
}

impl Luna {
    pub fn merge(&mut self, other: Self) {
        self.start = self.start.min(other.start);

        for program in other.programs {
            if let Some(self_program) = self.programs.iter_mut().find(|t| t.name == program.name) {
                self_program.merge(program);
            } else {
                self.programs.push(program);
            }
        }
    }

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
                        if let Some(domain) = asset.name.domain() {
                            self.insert_asset(Asset::from_str(host)?, None)?;
                            if let Some(pr) = self.program_by_asset(&domain) {
                                pr.assets.push(asset);
                                return Ok(());
                            }
                        }
                    }
                    return Err(format!("oos: {}", request.url).into());
                }

                AssetName::Subdomain(s) => {
                    if let Some(domain) = asset.name.domain() {
                        if let Some(pr) = self.program_by_asset(&domain) {
                            pr.assets.push(asset);
                            return Ok(());
                        }
                    }

                    return Err(format!("oos: {}", s).into());
                }

                AssetName::Domain(d) => {
                    if let Some(mut pr) = program {
                        if let Some(pr) = self.program_by_name(&pr.name) {
                            pr.assets.push(asset);
                        } else {
                            pr.assets.push(asset);
                            self.insert_program(pr)?;
                        }
                        return Ok(());
                    }
                    return Err(format!("oop: {}", d).into());
                }
                AssetName::Cidr(c) => {
                    if let Some(mut pr) = program {
                        if let Some(pr) = self.program_by_name(&pr.name) {
                            pr.assets.push(asset);
                        } else {
                            pr.assets.push(asset);
                            self.insert_program(pr)?;
                        }
                        return Ok(());
                    }
                    return Err(format!("oop: {}", c).into());
                }
            }
        }
        Ok(())
    }

    pub fn insert_tag(&mut self, tag: Tag, asset: &AssetName) -> Result<(), Errors> {
        if let Some(asset) = self.asset_by_name(asset) {
            asset.insert_tag(dbg!(tag));
            Ok(())
        } else if let Some(domain) = asset.domain() {
            if let Some(pr) = self.program_by_asset(&domain) {
                let asset = Asset {
                    name: asset.to_owned(),
                    tags: vec![tag],
                    start: Time::default(),
                };
                pr.assets.push(asset);
                Ok(())
            } else {
                Err("oos".into())
            }
        } else {
            Err("oop".into())
        }
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
            .filter(|a| filter.start.map_or(true, |t| t < a.start))
            .take(filter.n)
            .collect()
    }
    pub fn programs_mut(&mut self, filter: &Filter) -> Vec<&mut Program> {
        self.programs
            .iter_mut()
            .filter(|p| filter.program(p))
            .filter(|a| filter.start.map_or(true, |t| t < a.start))
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
            .filter(|a| filter.start.map_or(true, |t| t < a.start))
            .take(filter.n)
            .collect()
    }
    pub fn assets_mut(&mut self, field: Field, filter: &Filter) -> Vec<&mut Asset> {
        self.programs
            .iter_mut()
            .filter(|p| filter.program(p))
            .flat_map(|p| &mut p.assets)
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

    pub fn tags(&self, filter: &Filter) -> Vec<&Tag> {
        self.programs
            .iter()
            .filter(|p| filter.program(p))
            .flat_map(|p| &p.assets)
            .filter(|a| filter.asset(a))
            .flat_map(|a| &a.tags)
            .filter(|t| filter.tag(t))
            .filter(|a| filter.start.map_or(true, |t| t < a.start))
            .take(filter.n)
            .collect()
    }
    pub fn tags_mut(&mut self, filter: &Filter) -> Vec<&mut Tag> {
        self.programs
            .iter_mut()
            .filter(|p| filter.program(p))
            .flat_map(|p| &mut p.assets)
            .filter(|a| filter.asset(a))
            .flat_map(|a| &mut a.tags)
            .filter(|t| filter.tag(t))
            .filter(|a| filter.start.map_or(true, |t| t < a.start))
            .take(filter.n)
            .collect()
    }
    pub fn find(&self, field: Field, filter: &Filter, v: u8) -> Vec<String> {
        match field {
            Field::Luna => vec![self.stringify(v)],
            Field::Program => self
                .programs(filter)
                .iter()
                .map(|f| f.stringify(v))
                .collect(),
            Field::None => vec!["".to_string()],
            Field::Tag => self.tags(filter).iter().map(|f| f.stringify(v)).collect(),
            Field::Value => self.tags(filter).iter().map(|f| f.stringify(v)).collect(),
            _ => self
                .assets(field, filter)
                .iter()
                .map(|f| f.stringify(v))
                .collect(),
        }
    }

    pub fn remove(&mut self, field: Field, filter: &Filter) {
        match field {
            Field::Luna => error!("WTF!"),
            Field::Program => self.programs.retain(|p| !filter.program(p)),
            Field::None => debug!("!"),
            Field::Tag => self
                .assets_mut(Field::Asset, filter)
                .iter_mut()
                .for_each(|a| a.tags.retain(|t| !filter.tag(t))),
            Field::Value => self
                .tags_mut(filter)
                .iter_mut()
                .for_each(|t| t.values.retain(|v| !filter.value(v))),
            _ => self
                .programs_mut(filter)
                .iter_mut()
                .for_each(|p| p.assets.retain(|a| !filter.asset(a))),
        }
    }

    pub fn save_as(&self, path: &Path, backup: bool) -> Result<usize, Errors> {
        let str = serde_json::to_string(&self)?;

        if backup && path.exists() {
            let to = if let Some(ex) = path.extension() {
                format!(
                    "{}_{}.{}",
                    path.file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("luna"),
                    Local::now().to_rfc3339(),
                    ex.to_string_lossy()
                )
            } else {
                format!(
                    "{}_{}",
                    path.file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("luna"),
                    Local::now().to_rfc3339()
                )
            };

            std::fs::copy(&path, to)?;
        }
        match std::fs::File::options()
            .write(true)
            .truncate(true)
            .open(&path)
        {
            Ok(mut file) => Ok(file.write(str.as_bytes())?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Ok(std::fs::File::create(path)?.write(str.as_bytes())?)
            }
            Err(err) => Err(Box::new(err)),
        }
    }

    pub fn save(&self, path: &Path, backup: bool) {
        let output = path;

        if let Err(err) = self.save_as(output, backup) {
            error!("Error while saving: {}", err);
        } else {
            info!("Saved in \"{}\" successfully.", output.display());
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, Errors> {
        let file = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file)?)
    }

    pub fn parse(path: &Path) -> Luna {
        match Luna::from_file(path) {
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

    pub fn stringify(&self, v: u8) -> String {
        match v {
            0 => self.name.to_string(),
            1 => format!("{} {}", self.name, self.version),
            2 => format!(
                "{}  {}
    Status:   {}
    Programs: {}
    Assets:   {}
    Domains:  {}
    CIDRs:    {}
    Subs:     {}
    URLs:     {}
    Tags:     {}
    Start:    {}
    ",
                self.name,
                self.version,
                self.status,
                self.programs.len(),
                self.find(Field::Asset, &Filter::default(), 0).len(),
                self.find(Field::Domain, &Filter::default(), 0).len(),
                self.find(Field::Cidr, &Filter::default(), 0).len(),
                self.find(Field::Sub, &Filter::default(), 0).len(),
                self.find(Field::Url, &Filter::default(), 0).len(),
                self.find(Field::Tag, &Filter::default(), 0).len(),
                self.start
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
            ),
            3 => format!(
                "{}  {}
    Status:   {}
    Programs: [{}{}
    Assets:   {}
    Domains:  {}
    CIDRs:    {}
    Subs:     {}
    URLs:     {}
    Tags:     {}
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
                self.find(Field::Asset, &Filter::default(), 0).len(),
                self.find(Field::Domain, &Filter::default(), 0).len(),
                self.find(Field::Cidr, &Filter::default(), 0).len(),
                self.find(Field::Sub, &Filter::default(), 0).len(),
                self.find(Field::Url, &Filter::default(), 0).len(),
                self.find(Field::Tag, &Filter::default(), 0).len(),
                self.start
                    .0
                    .with_timezone(&Local::now().timezone())
                    .to_rfc2822(),
            ),
            _ => format!("{:#?}", self),
        }
    }
}
