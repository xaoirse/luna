use crate::{model, tools};
use clap::{load_yaml, App};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};

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

    // Establish connection or return None
    let mut conn = None;
    if let Some(db) = matches.value_of("db") {
        conn = model::Model::init(db).await.ok();
    }
    let conn = &mut conn;

    if let Some(ref matches) = matches.subcommand_matches("domain") {
        if let Some(n) = matches.value_of("a") {
            model::Model::Domain {
                name: n.to_string(),
            }
            .save(conn)
            .await;
        }
        if let Some(n) = matches.value_of("e") {
            model::Model::Domain {
                name: n.to_string(),
            }
            .exists(conn)
            .await;
        }
        if let Some(n) = matches.value_of("s") {
            tools::assetsfinder::run(conn, n, vec!["medsab.ac.ir".to_string()]).await;
        }
    }

    if let Some(ref matches) = matches.subcommand_matches("subdomain") {
        if let Some(n) = matches.value_of("a") {
            model::Model::Subdomain {
                name: n.to_string(),
                ip: "".to_string(),
            }
            .save(conn)
            .await;
        }
        if let Some(n) = matches.value_of("e") {
            model::Model::Subdomain {
                name: n.to_string(),
                ip: "".to_string(),
            }
            .exists(conn)
            .await;
        }
    }

    if let Some(ref matches) = matches.subcommand_matches("word") {
        if let Some(n) = matches.value_of("a") {
            model::Model::Word {
                name: n.to_string(),
                domain: "".to_string(),
            }
            .save(conn)
            .await;
        }
        if let Some(n) = matches.value_of("e") {
            model::Model::Word {
                name: n.to_string(),
                domain: "".to_string(),
            }
            .exists(conn)
            .await;
        }
    }

    // Chack exists
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
