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

pub fn ok<D>(text: D)
where
    D: Display,
{
    println!("{} {}", "[+]".green(), text.to_string().green())
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
    let cli = reqwest::Client::builder().build().unwrap();
    match cli
        .post("https://discord.com/api/webhooks/895773089936334908/78I0tKk9YgpyJuKqC7_NxjxOBgfqWF4PZp4Qksfl2KCTCQzHWvmicqF-7xM4pFjQ72e-")
        .header("Content-Type","application/json").body(format!(r#"{{"username": "Luna", "content": "{}"}}"#,text))
        .send()
        .await
    {
        Ok(_) => (),
        Err(err) =>crate::alert::nok(err),
    }
}
