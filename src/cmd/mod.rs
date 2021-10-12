use crate::model;
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

    if let Some(db) = matches.value_of("db") {
        let mut conn = model::Model::init(db).await.unwrap();

        // Save in both File and Database
        if let Some(n) = matches.value_of("ad") {
            model::Model::Domain { name: n }.save(&mut conn).await;
            model::save_file("domains.txt", n);
        }
        if let Some(n) = matches.value_of("as") {
            model::Model::Subdomain { name: n, ip: "" }
                .save(&mut conn)
                .await;
            model::save_file("subdomains.txt", n);
        }
        if let Some(n) = matches.value_of("aw") {
            model::Model::Word {
                name: n,
                domain: "",
            }
            .save(&mut conn)
            .await;
            model::save_file("wl-subdomains.txt", n);
        }
    } else {
        // Save in File
        if let Some(n) = matches.value_of("ad") {
            model::save_file("domains.txt", n);
        }
        if let Some(n) = matches.value_of("as") {
            model::save_file("subdomains.txt", n);
        }
        if let Some(n) = matches.value_of("aw") {
            model::save_file("wl-subdomains.txt", n);
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
