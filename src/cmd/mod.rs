use crate::{alert, env, model, tools};
use clap::{load_yaml, App};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use sqlx::any::AnyPoolOptions;

// strings
static BANNER: &str = r"
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.1.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

";
// static START: &str = "Luna is starting...";
// static DB_CHECKING: &str = "[ ] Checking DataBase ";
// static DB_CHECKED: &str = "[+] DataBase Checked  ";
// static DB_ERROR: &str = "[-] DataBase Failed   ";

pub async fn start() {
    // Welcome Banner
    println!("{}", BANNER.blue());

    // Extracting Parameters
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    // Establish connection to url or in-memory sqlite
    let pool;
    if let Some(url) = matches.value_of("db") {
        pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .unwrap();
    } else if let Some(url) = env::get("DATABASE") {
        pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .unwrap();
    } else {
        pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .unwrap();
    };
    model::init(&pool).await;

    // Subcommand Domain
    if let Some(ref matches) = matches.subcommand_matches("domain") {
        if let Some(n) = matches.value_of("a") {
            let r = model::Domain {
                name: n.to_string(),
                at: chrono::Utc::now().timestamp(),
            }
            .save(&pool)
            .await;
            if let Err(err) = r {
                // Duplicated shouldn't panic
                if err.as_database_error().unwrap().code().unwrap() == "23000"
                    || err.as_database_error().unwrap().code().unwrap() == "2067"
                {
                    alert::yok(format!("{} {}", n, "Exists!"));
                } else {
                    panic!(
                        "{} {}",
                        err.as_database_error().unwrap().code().unwrap(),
                        err
                    );
                }
            };
        }

        if let Some(n) = matches.value_of("f") {
            match model::Domain::fetch_optional(&pool, format!("name={}", n)).await {
                Some(d) => alert::found(d.to_string()),
                None => alert::nfound(n),
            }
        }
        if let Some(path) = matches.value_of("s") {
            let domains = model::Domain::fetch_all(&pool, "1=1".to_string()).await;
            tools::assetsfinder::run(&pool, path, domains).await;
        }
    }

    // Subcommand Subdomain
    if let Some(ref matches) = matches.subcommand_matches("subdomain") {
        if let Some(n) = matches.value_of("a") {
            let r = model::Subdomain {
                name: n.to_string(),
                ip: "".to_string(),
                at: chrono::Utc::now().timestamp(),
            }
            .save(&pool)
            .await;
            if let Err(err) = r {
                if err.as_database_error().unwrap().code().unwrap() == "23000" {
                    alert::yok(format!("{} {}", n, "Exists!"));
                } else {
                    panic!("{}", err);
                }
            };
        }
        if let Some(n) = matches.value_of("f") {
            match model::Subdomain::fetch_optional(&pool, format!("name={}", n)).await {
                Some(d) => alert::found(d.to_string()),
                None => alert::nfound(n),
            }
        }
    }

    // Subcommand Domain
    if let Some(ref matches) = matches.subcommand_matches("word") {
        if let Some(n) = matches.value_of("a") {
            let r = model::Word {
                name: n.to_string(),
                domain: "".to_string(),
                at: chrono::Utc::now().timestamp(),
            }
            .save(&pool)
            .await;
            if let Err(err) = r {
                if err.as_database_error().unwrap().code().unwrap() == "23000" {
                    alert::yok(format!("{} {}", n, "Exists!"));
                } else {
                    panic!("{}", err);
                }
            };
        }
        if let Some(n) = matches.value_of("f") {
            match model::Word::fetch_optional(&pool, format!("name={}", n)).await {
                Some(d) => alert::found(d.to_string()),
                None => alert::nfound(n),
            }
        }
    }
}

async fn _diff() {
    let diff = TextDiff::from_lines(
        "Hello World\nThis is the second line.\nThis is the third.",
        "Hallo Welt\nThis is the second line.\nThis is life.\nMoar and more",
    );

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}{}", sign, change);
    }

    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Some(i) = matches.value_of("INPUT") {
        println!("Value for input: {}", i);
    }
    if let Some(i) = matches.value_of("config") {
        println!("Value for config: {}", i);
    }
}
