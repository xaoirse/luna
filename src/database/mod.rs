pub mod mongo;

use crate::env;
use crate::model::*;
use mongodb::{options::ClientOptions, Client, Database};

pub async fn get_db_url() -> String {
    let path = match env::get("PATH") {
        Some(path) => path,
        None => ".".to_string(),
    };

    let url = match env::get("DATABASE") {
        Some(db_url) => db_url,
        None => {
            "No database detected!\nLuna will use in-memeory sqlite luna.db".warn();
            std::fs::OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(format!("{}/{}", path, "luna.db"))
                .unwrap();
            format!("sqlite://{}/{}", path, "luna.db")
        }
    };
    url
}
pub async fn get_db() -> Database {
    // Get db_url
    let url = get_db_url().await;

    // Parse a connection String, into an options
    let mut client_options = ClientOptions::parse(url).await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("Luna app".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // Return a handle to a database.
    if cfg!(test) {
        client.database("test")
    } else {
        client.database("luna")
    }
}

// pub async fn from_args() {
//     let url = get_db_url().await;

//     if url.starts_with("sql") {
//         model::action_from_args().await;
//     } else if url.starts_with("mongodb") {
//         let opt = model::Opt::from_args();
//         model::action_from_args(opt).await;
//     } else {
//         panic!("invalid Database url")
//     }
// }
