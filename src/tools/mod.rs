pub mod extractor;
pub mod file;

use async_trait::async_trait;
use std::process::Command;

use crate::alert::Alert;
use crate::model;
use extractor::{Extractor, Save};

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
        let mut data = std::collections::HashMap::new();
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
