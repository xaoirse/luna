use colored::Colorize;
use std::fmt::Display;

pub fn nok<D>(text: D)
where
    D: Display,
{
    let text = text.to_string();
    let text = text.trim();
    if !text.is_empty() {
        println!("{} {}", "[-]".red(), text.to_string().red())
    }
}

pub fn _ok<D>(text: D)
where
    D: Display,
{
    println!("{} {}", "[+]".green(), text.to_string().green())
}

pub fn yok<D>(text: D)
where
    D: Display,
{
    println!("{} {}", "[!]".yellow(), text.to_string().yellow())
}

pub fn found<D>(text: D)
where
    D: Display,
{
    println!("{} {}: Found.", "[+]".green(), text.to_string().green())
}
pub fn nfound<D>(text: D)
where
    D: Display,
{
    println!("{} {}: Not Found", "[+]".red(), text.to_string().red())
}

pub async fn push<D>(text: D)
where
    D: Display,
{
    if let Some(url) = crate::env::get("DISCORD") {
        let cli = reqwest::Client::builder().build().unwrap();
        match cli
            .post(url)
            .header("Content-Type", "application/json")
            .body(format!(r#"{{"username": "Luna", "content": "{}"}}"#, text))
            .send()
            .await
        {
            Ok(_) => (),
            Err(err) => crate::alert::nok(err),
        }
    }
}
