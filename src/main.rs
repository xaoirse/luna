// Luna
// Developed by SAoirse
// xaoirse.github.com

mod alert;
mod cmd;
mod env;
mod model;
// mod tests;
// mod tools;
use colored::Colorize;

static BANNER: &str = r"
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.4.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    
";

#[tokio::main]
async fn main() {
    println!("{}", BANNER.blue());

    // cmd::from_args().await;
    cmd::run::run().await;
}
