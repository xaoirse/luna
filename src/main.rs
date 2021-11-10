// Luna
// Developed by SAoirse
// xaoirse.github.com

mod alert;
mod env;
mod model;
mod tests;
mod tools;
use colored::Colorize;

static BANNER: &str = r"
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.2.1
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

";

#[tokio::main]
async fn main() {
    println!("{}", BANNER.blue());

    model::from_args().await;
}
