use crate::env;
pub mod mongo;
pub mod sql;
pub use crate::alert::Alert;

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

pub async fn from_args() {
    let url = get_db_url().await;

    if url.starts_with("sql") {
        sql::action_from_args().await;
    } else if url.starts_with("mongodb") {
        use structopt::StructOpt;
        let opt = mongo::Opt::from_args();
        mongo::action_from_args(opt).await;
    } else {
        panic!("invalid Database url")
    }
}
