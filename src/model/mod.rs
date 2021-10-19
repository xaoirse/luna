use orm::orm;
use sqlx::{any::AnyQueryResult, Any, Error, Pool};

// TODO trait with async fn

#[derive(orm, sqlx::FromRow)]
pub struct Domain {
    #[unique]
    pub name: String,
    pub at: i64,
}

#[derive(orm, sqlx::FromRow, Debug)]
pub struct Subdomain {
    #[unique]
    pub name: String,
    #[unique]
    pub ip: String,
    pub at: i64,
}

#[derive(orm, sqlx::FromRow)]
pub struct Word {
    #[unique]
    pub name: String,
    pub domain: String,
    pub at: i64,
}

// TODO some test that domain table has its fields
pub async fn init(pool: &Pool<Any>) {
    Domain::init(&pool).await.unwrap();
    Subdomain::init(&pool).await.unwrap();
    Word::init(&pool).await.unwrap();
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
