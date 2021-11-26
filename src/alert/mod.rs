use async_trait::async_trait;
use colored::Colorize;
use std::fmt::Display;

#[async_trait]
pub trait Alert {
    fn simple(&self);
    fn ok(&self);
    fn error(&self);
    fn warn(&self);
    fn found(&self);
    fn not_found(&self);
    async fn notif(self);
}

#[async_trait]
impl<T> Alert for T
where
    T: Display + Send,
{
    fn simple(&self) {
        let text = self.to_string();
        let text = text.trim();
        if !text.is_empty() {
            println!("{}", text)
        }
    }

    fn ok(&self) {
        let text = self.to_string();
        let text = text.trim();
        if !text.is_empty() {
            println!("{} {}", "[+]".green(), text.to_string().green())
        }
    }

    fn error(&self) {
        let text = self.to_string();
        let text = text.trim();
        if !text.is_empty() {
            println!("{} {}", "[-]".red(), text.red())
        }
    }

    fn warn(&self) {
        let text = self.to_string();
        let text = text.trim();
        if !text.is_empty() {
            println!("{} {}", "[!]".yellow(), text.to_string().yellow())
        }
    }
    fn found(&self) {
        let text = self.to_string();
        let text = text.trim();
        if !text.is_empty() {
            println!("{} {}: Found.", "[+]".green(), text.to_string().green())
        }
    }
    fn not_found(&self) {
        let text = self.to_string();
        let text = text.trim();
        if !text.is_empty() {
            println!("{} {}: Not Found", "[+]".red(), text.to_string().red())
        }
    }

    async fn notif(self) {
        let text = self.to_string();
        let text = text.trim();
        if !text.is_empty() {
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
                    Err(err) => err.error(),
                };
            }
        }
    }
}
