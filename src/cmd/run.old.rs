use crate::alert::Alert;
use crate::model::{Host, Sub, URL};
use crate::tools::{self, extractor::Extractor};
use futures::future::join_all;
use futures::{stream, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::join;

pub async fn run(entries: Vec<String>, script_name: String) {
    let script_name = Arc::new(script_name);
    let wl = Arc::new(Mutex::new(Vec::new()));

    // I used iter instead of par_iter but because map is lazy
    // at the end with join_all these run parallel (Hope)
    let e = entries.iter().map(|entry| {
        let mut key_vals = HashMap::new();
        key_vals.insert("$$", vec![entry.clone()]);

        // Run script
        let output = Arc::new(tools::run_script(key_vals, script_name.clone()));

        // Get future for running extractors
        let subs = tokio::task::spawn(join_all(output.clone().extract_fut(|mut t: Sub| {
            {
                wl.lock().unwrap().append(&mut t.wordlister());
            }
            t.scope = entry.clone();
            mongo::update(t)
        })));

        // let subs = tokio::task::spawn(output.clone().extract_and(|mut t: Sub| async {
        //     {
        //         wl.lock().unwrap().append(&mut t.wordlister());
        //     }
        //     t.scope = entry.clone();
        //     match mongo::update(t).await {
        //         Some(s) => s.asset.notif().await,
        //         None => false,
        //     };
        // }));

        let hosts = tokio::task::spawn(join_all(output.clone().extract_fut(|mut t: Host| {
            {
                wl.lock().unwrap().append(&mut t.wordlister());
            }
            t.sub = entry.clone();
            mongo::update(t)
        })));

        let urls = tokio::task::spawn(join_all(output.extract_fut(|mut t: URL| {
            {
                wl.lock().unwrap().append(&mut t.wordlister());
            }
            t.sub = entry.clone();
            mongo::update(t)
        })));

        // Gather all futures for Run all extractors in parallel
        tokio::spawn(async {
            let (_subs, _, _) = join!(subs, hosts, urls);
        })
    });

    // Run all extractors for all entries in parallel
    join_all(e).await;

    /////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////////////////////////////////////////////

    let e = entries.iter().map(|entry| {
        let mut key_vals = HashMap::new();
        key_vals.insert("$$", vec![entry.clone()]);

        // Run script
        let output = Arc::new(tools::run_script(key_vals, script_name.clone()));

        // Get future for running extractors
        let subs = tokio::task::spawn(join_all(output.clone().extract_fut(|mut t: Sub| {
            {
                wl.lock().unwrap().append(&mut t.wordlister());
            }
            t.scope = entry.clone();
            mongo::update(t)
        })));

        let entry = Arc::new(entry.to_owned());
        // let subs = stream::iter(output.clone().extract::<Sub>().into_iter())
        //     .map(|mut t| {
        //         let wl = wl.clone();
        //         let entry = entry.clone();

        //         async move {
        //             wl.clone().lock().unwrap().append(&mut t.wordlister());
        //             t.scope = entry.to_string();
        //             match mongo::update(t).await {
        //                 Some(s) => s.asset.notif().await,
        //                 None => false,
        //             };
        //         }
        //     })
        //     .buffer_unordered(2)
        //     .collect::<()>();

        let hosts = tokio::task::spawn(join_all(output.clone().extract_fut(|mut t: Host| {
            {
                wl.lock().unwrap().append(&mut t.wordlister());
            }
            t.sub = entry.to_string();
            mongo::update(t)
        })));

        let urls = tokio::task::spawn(join_all(output.extract_fut(|mut t: URL| {
            {
                wl.lock().unwrap().append(&mut t.wordlister());
            }
            t.sub = entry.to_string();
            mongo::update(t)
        })));

        // Gather all futures for Run all extractors in parallel
        tokio::spawn(async {
            // let (_subs, _, _) = join!(subs, hosts, urls);
        })
    });

    crate::tools::file::save("wl.txt", wl.lock().unwrap().to_vec());
}

pub async fn check() {
    // Check luna.ini exists
    match std::fs::read("luna.ini") {
        Ok(_) => "luna.ini".ok(),
        Err(err) => {
            "luna.ini".error();
            err.error();
        }
    }

    // Check database
    let db = crate::database::get_db().await;
    match db.list_collection_names(None).await {
        Ok(_) => {
            format!("Database is up!").ok();
        }
        Err(err) => err.error(),
    }

    // Check Discord
    let now = chrono::Local::now().to_string();
    if now.clone().notif().await {
        format!("Discord checked at: {}", &now).ok();
    } else {
        format!("Discord failed!").error();
    }
}

pub async fn run2(entries: Vec<String>, script_name: String) {
    let script_name = Arc::new(script_name);
    // let wl = Arc::new(Mutex::new(Vec::new()));

    let mut all_subs = Vec::new();
    let mut all_hosts = Vec::new();
    let mut all_urls = Vec::new();

    for entry in entries {
        let mut key_vals = HashMap::new();
        key_vals.insert("$$", vec![entry.clone()]);

        // Run script
        let output = Arc::new(tools::run_script(key_vals, script_name.clone()));

        // Extract structs
        let subs = output.clone().extract::<Sub>();
        let hosts = output.clone().extract::<Host>();
        let urls = output.clone().extract::<URL>();

        let (subs, hosts, urls) = join!(subs, hosts, urls);

        // Gather results
        if !subs.is_empty() {
            all_subs.push((entry.clone(), subs))
        }
        if !hosts.is_empty() {
            all_hosts.push((entry.clone(), hosts))
        }
        if !urls.is_empty() {
            all_urls.push((entry, urls))
        }
    }

    let report = report_generator(vec![]);

    println!("{}", report);
}

pub struct Item {
    title: String,
    items: Vec<String>,
}

fn report_generator(items: Vec<Item>) -> String {
    let header = format!("# Luna Reports  ");
    let start = format!("Started at: {}", chrono::Utc::now().to_rfc2822());
    let finish = format!("Finished at: {}", chrono::Utc::now().to_rfc2822());
    let sign = format!("Created by: Luna version 0.4.0");

    format!("{}\n{}\n{}\n{}", header, start, finish, sign)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn report_generator_test() {
        println!("{}", report_generator(vec![]));
    }
}
