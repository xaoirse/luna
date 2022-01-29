use futures::TryStreamExt;
use sqlx::{
    any::{AnyPoolOptions, AnyQueryResult},
    Any, Error, Executor, FromRow, Pool, Row,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    cli: Cli,
    // TODO
    // Server Setups
}
#[derive(Debug, StructOpt)]
pub enum Cli {
    Insert(Insert),
    Query(Query),
    Script(Script),
    Check,
    Test,
    Report,
}
#[derive(Debug, StructOpt)]
pub enum Insert {
    Scope(Scope),
}
#[derive(Debug, StructOpt)]
pub struct Query {
    query: String,
}
#[derive(Debug, StructOpt)]
pub struct Script {
    path: String,
}

#[derive(Debug, StructOpt, FromRow)]
pub struct Scope {
    #[structopt(long, default_value)]
    pub program: String,
    #[structopt(long, default_value)]
    pub scope: String,
    #[structopt(long, default_value)]
    pub sub: String,
}
impl Scope {
    fn new(scope: String) -> Self {
        Scope {
            scope,
            program: "".to_string(),
            sub: "".to_string(),
        }
    }
    async fn init(pool: &Pool<Any>) -> Result<AnyQueryResult, Error> {
        let query = r#"CREATE TABLE IF NOT EXISTS "Scopes" (
            "program"	TEXT,
            "scope"	    TEXT,
            "sub"	    TEXT  UNIQUE
        );"#;

        sqlx::query(query).execute(pool).await
    }

    async fn save(&self, pool: &Pool<Any>) -> Result<AnyQueryResult, Error> {
        sqlx::query("INSERT INTO Scopes(Program,Scope,Sub) VALUES(?,?,?)")
            .bind(self.program.clone())
            .bind(self.scope.clone())
            .bind(self.sub.clone())
            .execute(pool)
            .await
    }

    async fn fetch_one(self, pool: &Pool<Any>) -> Result<Self, Error> {
        let program = format!("%{}%", self.program);
        let scope = format!("%{}%", self.scope);
        let sub = format!("%{}%", self.sub);

        sqlx::query_as("SELECT Program,Scope,Sub FROM Scopes WHERE Program Like ? AND Scope Like ? AND Sub LIKE ?;")
            .bind(program)
            .bind(scope)
            .bind(sub)
            .fetch_one(pool)
            .await
    }
}
pub struct Sub {
    pub sub: String,
    pub url: Option<URL>,
    pub ip: Option<String>,
    pub services: Vec<Service>,
}
impl Sub {
    fn new(sub: String) -> Self {
        Sub {
            sub,
            url: None,
            ip: None,
            services: vec![],
        }
    }

    async fn init(pool: &Pool<Any>) -> Result<AnyQueryResult, Error> {
        let query = r#"CREATE TABLE IF NOT EXISTS "Subs" (
            "ID"	INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
            "Sub"	TEXT,
            "IP"	TEXT,
            "URL"	TEXT,
            "Tech"	TEXT,
            "Service"	TEXT
        );"#;

        sqlx::query(query).execute(pool).await
    }

    async fn save(&self, pool: &Pool<Any>) -> Result<AnyQueryResult, Error> {
        sqlx::query("sql")
            .bind(self.sub.clone())
            .execute(pool)
            .await
            .unwrap();
        todo!()
    }
}
pub struct Service {
    pub name: Option<String>,
    pub port: Option<String>,
    pub banner: Option<String>,
}

pub struct URL {
    pub url: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub content_type: Option<String>,
    pub techs: Option<String>,
}
pub struct Tech {
    pub name: Option<String>,
    pub version: Option<String>,
}

async fn init(pool: &Pool<Any>) -> Result<Vec<AnyQueryResult>, Error> {
    [Scope::init(pool).await, Sub::init(pool).await]
        .into_iter()
        .collect()
}

pub async fn get_pool(uri: &str, max: u32) -> Result<Pool<Any>, Error> {
    Ok(AnyPoolOptions::new()
        .max_connections(max)
        .connect(uri)
        .await?)
}
pub async fn run() {
    let pool = get_pool("sqlite://db.sqlite", 5).await.unwrap();
    init(&pool).await.unwrap();

    let opt = Opt::from_args();
    match opt.cli {
        Cli::Insert(insert) => match insert {
            Insert::Scope(s) => {
                s.save(&pool).await.unwrap();
            }
            _ => (),
        },
        Cli::Query(query) => {
            let s = Scope {
                program: "".to_string(),
                scope: query.query,
                sub: "".to_string(),
            }
            .fetch_one(&pool)
            .await
            .unwrap();
            dbg!(s);
        }
        Cli::Script(script) => (),
        _ => (),
    }
}
