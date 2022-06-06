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

    pub fn insert_program(&mut self, mut program: Program) -> Result<(), Errors> {
        for asset in std::mem::take(&mut program.assets) {
            if let Some(a) = self.asset_by_name(&asset.name) {
                a.merge(asset);
            } else {
                match program.assets.binary_search(&asset) {
                    Ok(i) => program.assets.get_mut(i).unwrap().merge(asset),
                    Err(i) => program.assets.insert(i, asset),
                }
            }
        }

        if let Some(p) = self.program_by_name(&program.name) {
            p.merge(program);
            p.aggregate();
        } else {
            program.aggregate();
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
                                let idx = pr.assets.binary_search(&asset).unwrap_or_else(|x| x);
                                pr.assets.insert(idx, asset);
                                return Ok(());
                            }
                        }
                    }
                    return Err(format!("OOS: {}", request.url).into());
                }

                AssetName::Subdomain(s) => {
                    if let Some(domain) = asset.name.domain() {
                        if let Some(pr) = self.program_by_asset(&domain) {
                            let idx = pr.assets.binary_search(&asset).unwrap_or_else(|x| x);
                            pr.assets.insert(idx, asset);
                            return Ok(());
                        }
                    }

                    return Err(format!("OOS: {}", s).into());
                }

                AssetName::Domain(d) => {
                    if let Some(mut pr) = program {
                        if let Some(pr) = self.program_by_name(&pr.name) {
                            let idx = pr.assets.binary_search(&asset).unwrap_or_else(|x| x);
                            pr.assets.insert(idx, asset);
                        } else {
                            let idx = pr.assets.binary_search(&asset).unwrap_or_else(|x| x);
                            pr.assets.insert(idx, asset);
                            self.insert_program(pr)?;
                        }
                        return Ok(());
                    }
                    return Err(format!("OOP: {}", d).into());
                }
                AssetName::Cidr(c) => {
                    if let Some(mut pr) = program {
                        if let Some(pr) = self.program_by_name(&pr.name) {
                            let idx = pr.assets.binary_search(&asset).unwrap_or_else(|x| x);
                            pr.assets.insert(idx, asset);
                            pr.aggregate();
                        } else {
                            let idx = pr.assets.binary_search(&asset).unwrap_or_else(|x| x);
                            pr.assets.insert(idx, asset);
                            pr.aggregate();
                            self.insert_program(pr)?;
                        }
                        return Ok(());
                    }
                    return Err(format!("OOP: {}", c).into());
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
                let idx = pr.assets.binary_search(&asset).unwrap_or_else(|x| x);
                pr.assets.insert(idx, asset);
                Ok(())
            } else {
                Err("OOS".into())
            }
        } else {
            Err("OOP".into())
        }
    }

    pub fn program_by_name(&mut self, name: &str) -> Option<&mut Program> {
        self.programs
            .iter_mut()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
    }
    pub fn program_by_asset(&mut self, asset: &AssetName) -> Option<&mut Program> {
        self.programs
            .iter_mut()
            .find(|p| p.assets.iter().any(|a| a.name.domain() == asset.domain()))
    }
    pub fn asset_by_name(&mut self, name: &AssetName) -> Option<&mut Asset> {
        let a = Asset {
            name: name.clone(),
            start: time::Time::default(),
            tags: vec![],
        };

        for p in &mut self.programs {
            if let Ok(i) = p.assets.binary_search(&a) {
                return p.assets.get_mut(i);
            }
        }
        None
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
            _ => self.programs_mut(filter).iter_mut().for_each(|p| {
                p.assets.retain(|a| match (field, &a.name) {
                    (Field::Domain, AssetName::Domain(_)) => !filter.asset(a),
                    (Field::Sub, AssetName::Subdomain(_)) => !filter.asset(a),
                    (Field::Url, AssetName::Url(_)) => !filter.asset(a),
                    (Field::Cidr, AssetName::Cidr(_)) => !filter.asset(a),
                    (Field::Asset, _) => !filter.asset(a),
                    _ => true,
                })
            }),
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
                    warn!("Can't load Luna from file! New file will be generated.")
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

mod test {
    use super::*;

    // https://github.com/rayon-rs/rayon/issues/592#issue-357025113
    #[test]
    fn insert() {
        let mut luna = Luna::default();
        let mut program = Program::from_str("test").unwrap();
        let asset = Asset::from_str("test.com").unwrap();
        let idx = program.assets.binary_search(&asset).unwrap_or_else(|x| x);
        program.assets.insert(idx, asset);
        luna.programs.push(program);

        let i = 0;
        let i = Mutex::new(i);

        let luna = Arc::new(Mutex::new(luna));

        for _ in 0..100 {
            (0..10).into_par_iter().for_each(|_| {
                let asset = Asset::from_str("a.test.com").unwrap();
                let mut i = i.lock().unwrap();

                println!("{} s", i);

                if let Err(err) = luna.lock().unwrap().insert_asset(asset, None) {
                    assert_eq!(err.to_string(), "OOS: a.test.com".to_string())
                };

                println!("{i} e");
                *i += 1;
            });
        }
    }

    #[allow(dead_code)]
    fn get_luna() -> Luna {
        let mut luna = Luna::default();

        let mut program = Program::from_str("google").unwrap();

        let mut asset = Asset::from_str("google.com").unwrap();

        let mut tag = Tag::from_str("sql").unwrap();

        tag.severity = Some("high".to_string());

        asset.tags.push(tag);

        let idx = program.assets.binary_search(&asset).unwrap_or_else(|x| x);
        program.assets.insert(idx, asset);

        luna.insert_program(program).unwrap();

        luna
    }
    #[test]
    fn find() {
        let luna = get_luna();

        let filter = Filter {
            severity: Some(filter::Regex::from_str("high").unwrap()),
            ..Default::default()
        };
        let res = luna.find(Field::Asset, &filter, 0);
        assert_eq!(res, vec!["google.com"]);

        let filter = Filter {
            severity: Some(filter::Regex::from_str("low").unwrap()),
            ..Default::default()
        };
        let res = luna.find(Field::Asset, &filter, 0);
        assert!(res.is_empty());
    }
}
