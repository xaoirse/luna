pub async fn scan() {
    let urls = std::fs::read_to_string("urls.txt").unwrap();
    let mut handles = Vec::new();
    for url in urls.lines() {
        let url = url.to_string();
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::builder().build().unwrap();
            match client.get(format!("http://{}", url)).send().await {
                Ok(resp) => println!(
                    "{} {} {}",
                    resp.status(),
                    resp.content_length().unwrap_or(0),
                    resp.url().to_string().green()
                ),
                Err(_) => println!("{}", url.to_string().red()),
            }
        });
        handles.push(handle)
    }
    let _ = futures::future::join_all(handles).await;
}
