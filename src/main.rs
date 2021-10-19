// Luna project is a web hunting app
// Developed by SAoirse

mod alert;
mod cmd;
mod env;
mod model;
mod tools;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
