use crate::alert;
use sqlx::{
    any::{Any, AnyArguments},
    query::Query,
    AnyConnection, Connection, Error,
};
mod file;

pub enum Model {
    Domain { name: String },
    Subdomain { name: String, ip: String },
    Word { name: String, domain: String },
}

impl Model {
    pub async fn init(url: &str) -> Result<AnyConnection, Error> {
        // TODO: switch for creating sqlite database if not exists
        // let conn = sqlx::sqlite::SqliteConnectOptions::from_str(url)?.create_if_missing(true);
        // let a = AnyConnectOptions::from(conn);
        // match AnyConnection::connect_with(&a).await {
        match AnyConnection::connect(url).await {
            Ok(mut conn) => {
                sqlx::query(
                    "
                    CREATE TABLE If NOT EXISTS domains (
                        name	TEXT UNIQUE,
	                    at	    INTEGER
                    );
                        ",
                )
                .execute(&mut conn)
                .await?;
                sqlx::query(
                    "
                    CREATE TABLE If NOT EXISTS subdomains (
	                    name	    TEXT,
                        ip      	TEXT,
	                    at	        INTEGER,
                        UNIQUE(name, ip)
                    );
                        ",
                )
                .execute(&mut conn)
                .await?;
                sqlx::query(
                    "
                    CREATE TABLE If NOT EXISTS wordlist (
                        name	    TEXT UNIQUE,
                        domain	    TEXT,
                        at	        INTEGER
                    );
                        ",
                )
                .execute(&mut conn)
                .await?;
                Ok(conn)
            }
            // Handling errors
            // err.as_database_error().unwrap().code().unwrap().eq("14")
            Err(err) => {
                alert::nok(format!("{} {}", "Database error: ", err));
                Err(err)
            }
        }
    }

    pub async fn save(&self, conn: &mut Option<AnyConnection>) {
        let mut is_new = false;
        let now = chrono::Utc::now().timestamp();
        let query: Query<Any, AnyArguments>;

        match self {
            Model::Domain { name } => {
                file::save("domains.txt", name);
                query = sqlx::query("insert into domains (name, at) values (?, ?)")
                    .bind(name)
                    .bind(now);
            }
            Model::Subdomain { name, ip } => {
                is_new = file::save("subdomains.txt", name);
                query = sqlx::query("insert into subdomains (name, ip, at) values (?, ?, ?)")
                    .bind(name)
                    .bind(ip)
                    .bind(now);
            }
            Model::Word { name, domain } => {
                file::save("wl.txt", name);
                query = sqlx::query("insert into wordlist (name, domain, at) values (?, ?, ?)")
                    .bind(name)
                    .bind(domain)
                    .bind(now);
            }
        }

        // Excute query if connection to database is estabilished
        if let Some(conn) = conn {
            match query.execute(conn).await {
                Ok(_) => is_new &= true,
                // If duplicated
                Err(err) if err.as_database_error().unwrap().code().unwrap() == "23000" => {
                    is_new = false
                }
                Err(err) => panic!("{}", err),
            }
        }

        // Alert if there is a new item
        if is_new {
            match self {
                Model::Subdomain { name, .. } => crate::alert::push(name).await,
                _ => (),
            }
        }
    }

    pub async fn exists(&self, conn: &mut Option<AnyConnection>) -> bool {
        let query: Query<Any, AnyArguments>;
        match self {
            Model::Domain { name } => {
                file::exists("domains.txt", name);
                query = sqlx::query("select * from domains where name=?").bind(name);
            }
            Model::Subdomain { name, .. } => {
                file::exists("subdomains.txt", name);
                query = sqlx::query("select * from subdomains where name=?").bind(name);
            }
            Model::Word { name, .. } => {
                file::exists("wl.txt", name);
                query = sqlx::query("select * from wordlist where name=?").bind(name);
            }
        }
        if let Some(conn) = conn {
            match query.fetch_optional(conn).await.unwrap() {
                Some(_) => {
                    alert::found("Database");
                    return true;
                }
                None => alert::nfound("Database"),
            }
        }

        false
    }

    pub async fn save_with_word(&self, conn: &mut Option<AnyConnection>) {
        match self {
            Model::Subdomain { name, .. } => {
                self.save(conn).await;
                for word in name.split(".") {
                    Model::Word {
                        name: word.to_string(),
                        domain: name.to_string(),
                    }
                    .save(conn)
                    .await;
                }
            }
            _ => (),
        }
    }

    // TODO
    // pub async fn _all(conn: &mut AnyConnection) -> Result<Vec<Domain>, sqlx::Error> {
    //     let hosts = sqlx::query_as::<_, Domain>("select * from domains")
    //         .fetch_all(conn)
    //         .await?;
    //     Ok(hosts)
    // }
}

pub fn subdomains_from_text(text: String) -> Vec<Model> {
    let mut subdomains = vec![];
    let regex = "((?:[a-z0-9A-Z]\\.)*[a-z0-9-]+\\.(?:[a-z0-9]{2,24})+(?:\\.co\\.(?:[a-z0-9]{2,24})|\\.(?:[a-z0-9]{2,24}))*)+.+(\\b(?:(?:2(?:[0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9])\\.){3}(?:(?:2([0-4][0-9]|5[0-5])|[0-1]?[0-9]?[0-9]))\\b)";
    let re = regex::Regex::new(&regex).unwrap();

    for v in re.captures_iter(&text) {
        subdomains.push(Model::Subdomain {
            name: v[1].to_string(),
            ip: v[2].to_string(),
        });
    }

    subdomains
}

// The code that i wrote it
//
// type Conn = Result<AnyConnection, sqlx::Error>;
// pub async fn add<'q, E, D>(conn: &mut Conn, query: E)
// where
//     D: sqlx::Database,
//     E: sqlx::Execute<'q, D> + sqlx::Execute<'q, sqlx::Any> + 'q,
// {
//     if let Ok(conn) = conn {
//         match conn.execute(query).await {
//             Ok(_) => (),
//             Err(_) => (),
//         }
//     }
// }
