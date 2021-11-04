// sqlx models
// todo: write complete models
// todo: write releations features
//

use crate::alert::Alert;
use orm::orm;
use serde::{Deserialize, Serialize};
use sqlx::{
    any::{Any, AnyPoolOptions, AnyQueryResult},
    Error, Pool,
};
use structopt::StructOpt;
// TODO trait with async fn

#[derive(orm, sqlx::FromRow, Debug, Serialize, Deserialize, StructOpt)]
pub struct Domain {
    #[structopt(short, long)]
    #[unique]
    pub name: String,
    #[structopt(skip)]
    pub at: i64,
}

#[derive(orm, sqlx::FromRow, Debug, Serialize, Deserialize, StructOpt)]
pub struct Subdomain {
    #[structopt(short, long)]
    #[unique]
    pub name: String,
    #[structopt(short, long)]
    #[unique]
    pub ip: String,
    #[structopt(skip)]
    pub at: i64,
}

#[derive(orm, sqlx::FromRow, Debug, Serialize, Deserialize, StructOpt)]
pub struct Word {
    #[structopt(short, long)]
    #[unique]
    pub name: String,
    #[structopt(short, long, default_value = "")]
    pub domain: String,
    #[structopt(skip)]
    pub at: i64,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "The Moon Rider has arrived.\nSQL")]
pub struct Opt {
    // #[structopt(short, long, help = "mysql://example.com/test")]
    // pub db_url: Option<String>,
    #[structopt(subcommand)]
    pub sub: Subcommand,
}
#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Insert(Insert),
    Scan(Scan),
}

#[derive(Debug, StructOpt)]
pub enum Insert {
    Domain(Domain),
    Subdomain(Subdomain),
    Word(Word),
}
#[derive(Debug, StructOpt)]
pub enum Scan {
    Domain(Domain),
    Subdomain(Subdomain),
}

// TODO some test that domain table has its fields

pub async fn get_db() -> Pool<Any> {
    let url = super::get_db_url().await;
    AnyPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap()
}
pub async fn action_from_args() {
    let pool = get_db().await;

    match Opt::from_args().sub {
        Subcommand::Insert(insert) => match insert {
            Insert::Domain(mut doc) => {
                Domain::init(&pool).await.unwrap();
                doc.at = chrono::Utc::now().timestamp();
                doc.save(&pool).await;
            }
            Insert::Subdomain(mut doc) => {
                Subdomain::init(&pool).await.unwrap();
                doc.at = chrono::Utc::now().timestamp();
                doc.save(&pool).await;
            }
            Insert::Word(mut doc) => {
                Word::init(&pool).await.unwrap();
                doc.at = chrono::Utc::now().timestamp();
                doc.save(&pool).await;
            }
        },
        _ => {}
    }
}

pub fn _utc_days_ago() -> String {
    "".to_string()
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

// TODO TESTS
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use sqlx::Connection;
//     #[derive(orm, sqlx::FromRow, Debug)]
//     struct Test {
//         #[unique]
//         name: String,
//         #[integer]
//         #[unique]
//         id: String,
//     }
//     #[tokio::test]
//     async fn test_orm_macro_1() {
//         let mut pool = AnyConnection::connect("sqlite::memory:").await.unwrap();
//         Test::init(&mut pool).await.unwrap();
//         let t1 = Test {
//             name: "mia".to_string(),
//             id: "plays".to_string(),
//         };
//         t1.save(&mut pool).await.unwrap();
//         let t2 = Test::fetch_optional(&mut pool, "name = 'mia' AND id = 'plays'")
//             .await
//             .unwrap();
//         assert_eq!(t2.name, t1.name);
//         assert_eq!(1, Test::fetch_all(&mut conn, "1=1").await.len());
//         Test::update(&mut pool, "name='kimia'", "1=1")
//             .await
//             .unwrap();
//         let t2 = Test::fetch_optional(&mut pool, "name='kimia' AND id='plays'")
//             .await
//             .unwrap();
//         assert_eq!(t2.name, "kimia");
//         assert_eq!(1, Test::fetch_all(&mut pool, "1=1").await.len());

//         Test::delete(&mut pool, "name = 'kimia'").await.unwrap();
//         assert_eq!(0, Test::fetch_all(&mut pool, "1=1").await.len());
//     }

//     #[tokio::test]
//     #[should_panic]
//     async fn test_orm_macro_2() {
//         let mut pool = AnyConnection::connect("sqlite::memory:").await.unwrap();
//         Test::init(&mut pool).await.unwrap();
//         let t1 = Test {
//             name: "mia".to_string(),
//             id: "plays".to_string(),
//         };
//         t1.save(&mut pool).await.unwrap();
//         t1.save(&mut pool).await.unwrap();
//     }

//     #[derive(orm, sqlx::FromRow, Debug)]
//     struct Test2 {
//         name: String,
//         id: String,
//     }

//     #[tokio::test]
//     async fn test_orm_macro_3() {
//         let mut pool = AnyConnection::connect("sqlite::memory:").await.unwrap();
//         Test2::init(&mut conn).await.unwrap();
//         let t1 = Test2 {
//             name: "mia".to_string(),
//             id: "plays".to_string(),
//         };
//         t1.save(&mut conn).await.unwrap();
//         let t2 = Test2::fetch_optional(&mut conn, "name = 'mia' AND id = 'plays'")
//             .await
//             .unwrap();
//         assert_eq!(t2.name, t1.name);
//         assert_eq!(1, Test2::fetch_all(&mut conn, "1=1").await.len());
//         Test2::update(&mut conn, "name='kimia'", "name='mia' AND id ='plays'")
//             .await
//             .unwrap();
//         let t2 = Test2::fetch_optional(&mut conn, "name='kimia' AND id='plays'")
//             .await
//             .unwrap();
//         assert_eq!(t2.name, "kimia");
//         assert_eq!(1, Test2::fetch_all(&mut conn, "1=1").await.len());

//         Test2::delete(&mut conn, "name = 'kimia'").await.unwrap();
//         assert_eq!(0, Test2::fetch_all(&mut conn, "1=1").await.len());
//     }
//     #[tokio::test]
//     async fn test_orm_macro_4() {
//         let mut pool = AnyConnection::connect("sqlite::memory:").await.unwrap();
//         Test2::init(&mut conn).await.unwrap();
//         let t1 = Test2 {
//             name: "mia".to_string(),
//             id: "plays".to_string(),
//         };
//         t1.save(&mut conn).await.unwrap();
//         t1.save(&mut conn).await.unwrap();
//         assert_eq!(2, Test2::fetch_all(&mut conn, "name = 'mia'").await.len());
//     }
// }
