use super::*;

fn parse(text: &str, regex: &Regex) -> Vec<Asset> {
    regex
        .captures_iter(text)
        .filter_map(|caps| {
            let get = |key| caps.name(key).map(|v| v.as_str().to_string());

            let tags = if let Some(name) = get("tag") {
                let values = if let Some(value) = get("value") {
                    value.split(',').map(|s| s.to_string()).collect()
                } else {
                    vec![]
                };

                name.split(',')
                    .map(|name| Tag {
                        name: name.to_string(),
                        severity: get("severity"),
                        values: values.clone(),
                        ..Default::default()
                    })
                    .collect()
            } else {
                vec![]
            };

            if let Some(name) = get("asset") {
                if let Ok(mut name) = AssetName::from_str(&name) {
                    if let AssetName::Url(req) = &mut name {
                        req.title = get("title");
                        req.sc = get("sc");
                        req.resp = get("resp");
                    }
                    Some(Asset {
                        name,
                        tags,
                        start: Time(Utc::now()),
                    })
                } else {
                    warn!("Invalid asset: {}", name);
                    None
                }
            } else {
                warn!("No asset name!");
                None
            }
        })
        .collect()
}

pub struct Script {
    pub verbose: u8,
    pub cd: String,
    pub regex: Regex,
    pub command: String,
    pub field: Field,
}

impl Script {
    fn execute(&self, luna: &mut Luna, filter: &Filter, term: Arc<AtomicBool>) {
        debug!("{}", self.command);

        let elements = luna.find(self.field, filter, 0);

        let ps = ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} {wide_msg:.green}\n{elapsed_precise:.yellow} {wide_bar:.cyan/cyan} {pos:}/{len:} {eta:.magenta}",
                    ).unwrap()
                    .progress_chars("▓█░");

        let pb = ProgressBar::new(elements.len() as u64);

        pb.set_style(ps);

        if self.verbose == 0 {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        } else {
            pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
        }

        if self.verbose > 1 {
            pb.clone()
                .with_finish(ProgressFinish::WithMessage(self.command.clone().into()));
        }

        let luna = Mutex::new(luna);

        elements.par_iter().for_each(|input| {
            if term.load(atomic::Ordering::Relaxed) {
                warn!("Command aborted! {} => {}", input, self.command);
                return;
            }

            let cmd = self.command.replace(&self.field.substitution(), input);
            debug!("Command: {}", &cmd);

            pb.set_message(cmd.clone());

            let mut child = match Command::new("sh")
                .current_dir(&self.cd)
                .arg("-c")
                .arg(&cmd)
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(err) => {
                    error!("{err}");
                    return;
                }
            };

            match child.stdout.as_mut() {
                Some(stdout) => {
                    let stdout_reader = BufReader::new(stdout);
                    let stdout_lines = stdout_reader.lines();

                    for line in stdout_lines {
                        if term.load(atomic::Ordering::Relaxed) {
                            warn!("Command aborted while reading stdout!");
                            return;
                        }
                        match line {
                            Ok(line) => {
                                let assets = parse(&line, &self.regex);

                                debug!("Stdout assets len: {} {}", &assets.len(), cmd);

                                for asset in assets {
                                    debug!("Insert: {}", asset.stringify(2));
                                    if let Err(err) = luna.lock().unwrap().insert_asset(asset, None)
                                    {
                                        warn!("{err}");
                                    };
                                }
                            }
                            Err(err) => {
                                warn!("Error while reading lines from stdout: {err} {cmd}")
                            }
                        }
                    }
                }
                None => debug!("There is no stdout: {cmd}"),
            }

            match child.wait() {
                Ok(ok) => {
                    debug!("Success Command with StatusCode {ok}: {cmd}");
                    pb.inc(1);
                }
                Err(err) => debug!("Error in Waiting for command: {cmd} {err}"),
            }
        });
    }
}

pub struct Scripts {
    pub scripts: Vec<Script>,
    pub filter: Filter,
}

impl Scripts {
    pub fn run(self, luna: &mut Luna, path: &Path, backup: bool, term: Arc<AtomicBool>) {
        for script in self.scripts {
            if term.load(atomic::Ordering::Relaxed) {
                return;
            }
            script.execute(luna, &self.filter, term.clone());

            luna.save(path, backup);
        }
    }
}

#[derive(Parser)]
pub struct ScriptCli {
    pub path: std::path::PathBuf,
    #[clap(short, long, parse(from_occurrences), help = "Show progress bar")]
    pub verbose: u8,
    #[clap(flatten)]
    pub filter: Filter,
}

impl ScriptCli {
    #[allow(clippy::blocks_in_if_conditions)]
    pub fn parse(self) -> Result<Scripts, Errors> {
        let mut scripts = vec![];
        let mut regex = String::new();
        let regex_pat = Regex::new(r"(?:^#\s)*regex\s*=")?;

        for (n, line) in std::fs::read_to_string(&self.path)?
            .trim()
            .lines()
            .enumerate()
        {
            if regex_pat.is_match(line) {
                regex = line
                    .split_once('=')
                    .map_or("".to_string(), |p| p.1.trim().to_string())
            } else if line.trim().chars().next().map_or(false, |c| {
                c.is_ascii_alphabetic() || '.' == c || '/' == c || '\\' == c
            }) {
                if regex.is_empty() {
                    return Err("Where the fuck is the first regex?".into());
                }

                let field = if line.contains("${program}") {
                    Field::Program
                } else if line.contains("${domain}") {
                    Field::Domain
                } else if line.contains("${cidr}") {
                    Field::Cidr
                } else if line.contains("${sub}") {
                    Field::Sub
                } else if line.contains("${url}") {
                    Field::Url
                } else if line.contains("${tag}") {
                    Field::Tag
                } else if line.contains("${value}") {
                    Field::Value
                } else {
                    Field::None
                };

                if let Ok(regex) = Regex::new(&regex) {
                    if !is_valid(&regex) {
                        return Err(
                        format!("Line {} regex \"{}\"  Doesn't have necessery names \"asset\" or \"tag\""
                            ,n,regex).into(),
                    );
                    }

                    let mut cd = std::path::Path::new(&self.path)
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    if cd.is_empty() {
                        cd = ".".to_string()
                    }

                    let script = Script {
                        verbose: self.verbose,
                        cd,
                        regex,
                        command: line.trim().to_string(),
                        field,
                    };
                    scripts.push(script)
                } else {
                    error!("Fucking regex: {}", regex);
                    panic!("Fucking regex: {}", regex);
                }
            }
        }

        Ok(Scripts {
            scripts,
            filter: self.filter,
        })
    }
}

fn is_valid(regex: &Regex) -> bool {
    regex.capture_names().len() == 0
        || (regex.capture_names().flatten().any(|x| x == "asset")
            && (regex.capture_names().flatten().any(|x| x == "tag")
                || !(regex.capture_names().flatten().any(|x| x == "severity")
                    || regex.capture_names().flatten().any(|x| x == "value"))))
}
