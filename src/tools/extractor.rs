use async_trait::async_trait;

type DomainIps = Vec<(String, String)>;

#[async_trait]
pub trait Save {
    async fn save(self, scope: String);
}
#[async_trait]
impl Save for DomainIps {
    async fn save(mut self, scope: String) {
        self.sort();
        self.dedup();
        // TODO
    }
}

pub trait Extractor {
    fn subdomains(self) -> DomainIps;
}
impl Extractor for String {
    fn subdomains(self) -> DomainIps {
        let text = self.to_string();
        let mut subdomains = vec![];
        let regex = r"((?:[0-9\-a-z]+\.)+[a-z]+)(?:$|[\D\W]+)((?:[0-9]{1,3}\.){3}[0-9]{1,3})?(?:$|[\D\W\s])";
        let re = regex::RegexBuilder::new(&regex)
            .multi_line(true)
            .build()
            .unwrap();
        for text in text.lines() {
            for v in re.captures_iter(&text) {
                let name = v[1].to_string();
                let ip = match &v.get(2) {
                    Some(m) => m.as_str().to_string(),
                    None => "".to_string(),
                };
                subdomains.push((name, ip));
            }
        }

        subdomains
    }
}
