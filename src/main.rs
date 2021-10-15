// Luna project is a web hunting app
// SAoirse

mod alert;
mod cmd;
mod model;
mod tools;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
