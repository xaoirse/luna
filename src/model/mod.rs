use sqlx::{
    any::{Any, AnyArguments},
    query::Query,
    AnyConnection, Connection,
};
pub enum Model<'a> {
    Domain { name: &'a str },
    Subdomain { name: &'a str, ip: &'a str },
    Word { name: &'a str, domain: &'a str },
}

impl<'a> Model<'a> {
    pub async fn init(url: &str) -> Result<AnyConnection, sqlx::Error> {
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
                        name	    TEXT,
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
            Err(err) => Err(err),
        }
    }

    pub async fn save(&self, conn: &mut AnyConnection) {
        let now = chrono::Utc::now().timestamp();
        let query: Query<Any, AnyArguments>;
        match self {
            Model::Domain { name } => {
                query = sqlx::query("insert into domains (name, at) values (?, ?)")
                    .bind(name)
                    .bind(now);
            }
            Model::Subdomain { name, ip } => {
                query = sqlx::query("insert into subdomains (name, ip, at) values (?, ?, ?)")
                    .bind(name)
                    .bind(ip)
                    .bind(now);
            }
            Model::Word { name, domain } => {
                query = sqlx::query("insert into wordlist (name, domain, at) values (?, ?, ?)")
                    .bind(name)
                    .bind(domain)
                    .bind(now);
            }
        }
        match query.execute(conn).await {
            Ok(_) => (),
            Err(err) if err.as_database_error().unwrap().code().unwrap() == "23000" => (),
            Err(err) => panic!("{}", err),
        };
    }

    // pub async fn _all(conn: &mut AnyConnection) -> Result<Vec<Domain>, sqlx::Error> {
    //     let hosts = sqlx::query_as::<_, Domain>("select * from domains")
    //         .fetch_all(conn)
    //         .await?;
    //     Ok(hosts)
    // }
}

use crate::mylog;
use dotenv_codegen::*;
use std::io::{Read, Write};

static PATH: &str = dotenv!("DOMAINS_PATH");
pub fn save_file(file_name: &str, text: &str) {
    match std::fs::OpenOptions::new()
        .append(true)
        .read(true)
        .create(true)
        .open(format!("{}/{}", PATH, file_name))
    {
        Ok(mut file) => {
            let mut f = String::new();
            file.read_to_string(&mut f).unwrap();
            for l in f.lines() {
                if l == text {
                    return;
                }
            }
            file.write(text.as_bytes()).unwrap();
            file.write(b"\n").unwrap();
        }
        Err(err) => mylog::nok(&err.to_string()),
    }
}

// The code that i write it
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
