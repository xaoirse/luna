pub mod extractor;
mod file;

use std::{collections::HashMap, process::Command};

use crate::alert::Alert;
use file::Commands;

trait ToString {
    fn to_string(self) -> String;
}
impl ToString for Vec<u8> {
    fn to_string(self) -> String {
        String::from_utf8(self).unwrap()
    }
}

pub fn run_script(key_vals: HashMap<String, Vec<String>>, script_name: &str) -> String {
    key_vals
        .commands(script_name)
        .iter()
        .map(|command| {
            let std = Command::new("sh").arg("-c").arg(command).output().unwrap();
            std.stderr.to_string().error();

            // std.stdout.clone().to_string().simple(); // For debug

            std.stdout.to_string()
        })
        .collect::<String>()
}

// pub async fn _scan() {
//     let urls = std::fs::read_to_string("urls.txt").unwrap();
//     let mut handles = Vec::new();
//     for url in urls.lines() {
//         let url = url.to_string();
//         let handle = tokio::spawn(async move {
//             let client = reqwest::Client::builder().build().unwrap();
//             match client.get(format!("http://{}", url)).send().await {
//                 Ok(resp) => alert::_ok(format!(
//                     "{} {} {}",
//                     resp.status(),
//                     resp.content_length().unwrap_or(0),
//                     resp.url().to_string()
//                 )),
//                 Err(_) => alert::nok(url.to_string()),
//             }
//         });
//         handles.push(handle)
//     }
//     let _ = futures::future::join_all(handles).await;
// }
