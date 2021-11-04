use async_trait::async_trait;
use std::process::Command;

use super::extractor::{Extractor, Save};
use super::file::save;
use crate::alert::Alert;
use crate::model;
use crate::tools::Merge;
trait ToString {
    fn to_string(self) -> String;
}
impl ToString for Vec<u8> {
    fn to_string(self) -> String {
        String::from_utf8(self).unwrap()
    }
}

#[async_trait]
pub trait UpdateAssets {
    async fn update_assets(&self);
}
#[async_trait]
impl UpdateAssets for model::mongo::Scope {
    async fn update_assets(&self) {
        let mut results = Vec::new();

        // Data for replace with $keywords in commands
        let mut data: super::Data = std::collections::HashMap::new();
        data.insert("$domain".to_string(), vec![self.asset.clone()]);

        data.commands().iter().for_each(|command| {
            // Run command and print stderr if isn't empty
            let std = Command::new("sh").arg("-c").arg(command).output().unwrap();
            std.stderr.to_string().error();
            // For debug
            // std.stdout.clone().ok();

            // Extract all subdomains from stdout with regex
            results.append(&mut std.stdout.to_string().subdomains());
        });
        results.save(self.asset.clone()).await;
    }
}

/*
pub fn subdomains_from_text(text: String) -> Vec<(String, String)> {
    let mut subdomains = vec![];
    let regex =
        r"((?:[0-9\-a-z]+\.)+[a-z]+)(?:$|[\D\W]+)((?:[0-9]{1,3}\.){3}[0-9]{1,3})?(?:$|[\D\W\s])";
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


pub async fn _run(conn: &sqlx::Pool<sqlx::Any>, path: &str, domains: Vec<model::sql::Domain>) {
    // Map domains to their names
    let domains = domains.into_iter().map(|f| f.name).collect::<Vec<String>>();

    // Data for replace with $keywords in commands
    let mut data: super::Data = std::collections::HashMap::new();
    data.insert("$domain".to_string(), domains);
    let commands = parse_file(path, data);

    // Run each command and save results
    for cmd in commands {
        // Run command and print stderr if isn't empty
        let std = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
        alert::nok(String::from_utf8(std.stderr).unwrap());
        // For debug
        // println!("{}", String::from_utf8(std.stdout.clone()).unwrap());

        // Extract all subdomains from stdout with regex
        let subdomains = subdomains_from_text(String::from_utf8(std.stdout).unwrap());

        // Save new subdomains and update wordlist in database and file
        for sd in subdomains {
            match sd.save(conn).await {
                true => {
                    save("results.txt", &sd.to_string());
                    alert::push(sd.to_string()).await;
                }
                false => (),
            }
            for word in sd.name.split(".") {
                let w = model::sql::Word {
                    name: word.to_string(),
                    domain: sd.name.to_string(),
                    at: chrono::Utc::now().timestamp(),
                };
                match w.save(conn).await {
                    true => {
                        save("wl.txt", &w.name.to_string());
                    }
                    false => (),
                };
            }
        }

        // TODO make it parallel
        // use futures::future::join3;
        // use tokio::sync::mpsc;
        // let (tx, mut rx) = mpsc::channel::<String>(100);
        // let sd = tokio::spawn(async move {
        //     while let Some(message) = rx.recv().await {
        //         save("results.txt", message.as_str());
        //     }
        // });

        // let (txw, mut rxw) = mpsc::channel::<String>(100);
        // let w = tokio::spawn(async move {
        //     while let Some(message) = rxw.recv().await {
        //         save("wl.txt", message.as_str());
        //     }
        // });
        // let conn = conn.clone();
        // let r = tokio::spawn(async move {
        //     for sd in subdomains {
        //         match sd.save(&conn).await {
        //             Ok(_) => {
        //                 tx.send(sd.to_string()).await;
        //                 alert::push(sd.to_string()).await;
        //             }
        //             Err(_) => (),
        //         }
        //         for word in sd.name.split(".") {
        //             let w = model::Word {
        //                 name: word.to_string(),
        //                 domain: sd.name.to_string(),
        //                 at: chrono::Utc::now().timestamp(),
        //             };
        //             match w.save(&conn.clone()).await {
        //                 Ok(_) => {
        //                     txw.send(sd.to_string()).await;
        //                     alert::push(sd.to_string()).await;
        //                 }
        //                 Err(_) => (),
        //             }
        //         }
        //     }
        // });
        // join3(r, w, sd).await;
    }
}


pub fn _subdomains_from_text(text: String) -> Vec<model::sql::Subdomain> {
    let mut subdomains = vec![];
    let regex =
        r"((?:[0-9\-a-z]+\.)+[a-z]+)(?:$|[\D\W]+)((?:[0-9]{1,3}\.){3}[0-9]{1,3})?(?:$|[\D\W\s])";
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
            subdomains.push(model::sql::Subdomain {
                name,
                ip,
                at: chrono::Utc::now().timestamp(),
            });
        }
    }

    subdomains
}

pub async fn _scan() {
    let urls = std::fs::read_to_string("urls.txt").unwrap();
    let mut handles = Vec::new();
    for url in urls.lines() {
        let url = url.to_string();
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::builder().build().unwrap();
            match client.get(format!("http://{}", url)).send().await {
                Ok(resp) => alert::_ok(format!(
                    "{} {} {}",
                    resp.status(),
                    resp.content_length().unwrap_or(0),
                    resp.url().to_string()
                )),
                Err(_) => alert::nok(url.to_string()),
            }
        });
        handles.push(handle)
    }
    let _ = futures::future::join_all(handles).await;
}
*/
