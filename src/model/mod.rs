use async_trait::async_trait;
use mongodb::bson::Document;
use regex::Regex;

pub mod host;
pub mod job;
pub mod program;
pub mod scope;
pub mod service;
pub mod sub;
pub mod tech;
pub mod url;
pub use crate::alert::Alert;

pub use host::Host;
pub use program::Program;
pub use scope::Scope;
pub use service::Service;
pub use sub::Sub;
pub use tech::Tech;
pub use url::URL;

#[async_trait]
pub trait Model {
    fn id_query(&self) -> Document;
    fn wordlister(&self) -> Vec<String>;
    async fn merge(self, doc: Self) -> Self;
    fn new(id: String, parent: String) -> Self;
    fn ident() -> String;
    fn regex() -> Regex;
}
