use std::process::Command;

use crate::alert;
use crate::model;
use crate::tools::parse_file;

pub async fn run(conn: &mut Option<sqlx::AnyConnection>, path: &str, domains: Vec<String>) {
    // Data for replace with $keywords in commands
    let mut data: super::Data = std::collections::HashMap::new();
    data.insert("$domain".to_string(), domains);
    data.insert("$dns".to_string(), vec!["ns1.medsab.ac.ir".to_string()]);
    let commands = parse_file(path, data);

    // Run each command and save results
    for cmd in commands {
        let std = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
        alert::nok(String::from_utf8(std.stderr).unwrap());
        let subdomains = model::subdomains_from_text(String::from_utf8(std.stdout).unwrap());

        for sd in subdomains {
            sd.save_with_word(conn).await;
        }
    }
}

pub async fn _scan() {
    let urls = std::fs::read_to_string("urls.txt").unwrap();
    let mut handles = Vec::new();
    for url in urls.lines() {
        let url = url.to_string();
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::builder().build().unwrap();
            match client.get(format!("http://{}", url)).send().await {
                Ok(resp) => alert::ok(format!(
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
